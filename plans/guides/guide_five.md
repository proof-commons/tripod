#  Guide 5

## Overall assessment

Guide 5 is the correct next step **after Guide 4 has reaccepted the proof-plan candidate set**.

Its purpose should be narrower than “start lowering toward tapscript.” It should answer three target-independent questions:

1. **In which semantic execution cases is each relation active?**
2. **What abstract carrier roles could enforce each active runtime relation?**
3. **What layout and fact-availability requirements must a future backend satisfy for those carriers to work?**

The central pipeline should be:

```text
validated compiler input
    ↓
feasible proof-plan candidates
    ↓
typed execution cases
    ↓
relation × case discharge requirements
    ↓
eligible abstract carrier roles
    ↓
exact feasible placement sets
    ↓
target-independent layout requirements
```

Guide 5 should **not** emit a target program, choose transaction indexes, assign
tapleaves, freeze an ABI, or claim target enforcement.

The desired result is a complete, deterministic set of **requirements for a
future backend**, not a partially implemented backend.

---

# 1. The most important conceptual correction

The existing planning language says relations should be classified as:

```text
local
transaction-global
conditional
deliberately duplicated
```

Those are not four mutually exclusive categories.

They are different axes:

| Axis | Example values |
|---|---|
| discharge boundary | compiler-static, backend-structural, runtime, external evidence |
| semantic scope | member-local, family-global, transaction-global |
| activation | always, sponsor-present, selected representation, branch condition |
| multiplicity | exactly one carrier, every member, at least one, deliberate duplication |

For example, live-receipt owner authorization is:

```text
discharge boundary:
    runtime

semantic scope:
    member-local

activation:
    every live-transfer case

multiplicity:
    every consumed live-receipt member
```

Sponsor isolation is:

```text
discharge boundary:
    runtime plus substrate evidence

semantic scope:
    transaction-global

activation:
    all cases, with sponsor-specific source requirements active only when present

multiplicity:
    one complete global carrier
```

Representation is:

```text
discharge boundary:
    compiler-static selection
    plus backend-structural encoding requirement

semantic scope:
    object-family global

activation:
    selected representation mode

multiplicity:
    not an independent arithmetic carrier
```

Guide 5 should encode these dimensions separately. A single enum such as:

```rust
enum PlacementClass {
    Local,
    Global,
    Conditional,
    Duplicated,
}
```

would be too weak and would create contradictions later.

---

# 2. Guide 5’s precise scope

Guide 5 should implement:

```text
C1-009
P2-010
```

It should include:

- typed execution-case identities;
- proof-plan-specific case derivation;
- relation/case discharge classification;
- abstract semantic carrier roles;
- carrier eligibility based on fact and witness availability;
- exact placement enumeration;
- target-independent layout requirements;
- deterministic stable projections;
- explicit search limits;
- an independent exhaustive placement oracle;
- complete analysis for the two Phase-1 pilots.

It should stop before:

```text
C1-010
P2-011
P2-012
```

In particular, it should not yet implement:

- general symbol resolution;
- accepted cycle/SCC strategies;
- relation-indexed vector coverage;
- a complete analyzed-program public value;
- target capability mapping;
- backend emission;
- concrete transaction layout.

---

# 3. Guide 5 should consume proof-plan candidates, not bypass them

Placement is plan-specific.

A live-transfer proof plan selecting:

```text
PrivateCommitted
+
ConfidentialConservation
```

has different source and target requirements from one selecting:

```text
Explicit
+
PublicArithmetic
```

Therefore placement must consume each complete feasible proof-plan candidate,
not rebuild proof decisions independently.

Conceptually:

```rust
struct PlacedProofPlanCandidate {
    proof_plan: ProofPlanCandidate,
    execution_cases: Vec<ExecutionCase>,
    relation_case_plans: Vec<RelationCasePlan>,
    feasible_placements: Vec<PlacementCandidate>,
    layout_requirements: Vec<LayoutRequirement>,
}
```

These types can remain crate-private until P2-012 constructs a complete
analyzed program.

The implementation must not refer to proof plans by:

- vector position;
- search order;
- Petgraph index;
- candidate number;
- hash.

Until an admitted identity exists, the complete typed proof-plan value is the
boundary.

---

# 4. Execution-case model

## 4.1 What an execution case means

An execution case is not a sample transaction and not a vector fixture.

It is a finite semantic equivalence class under which:

- the same relations are active;
- the same source requirements are active;
- the same representation is selected;
- the same carrier-presence assumptions hold;
- the same placement constraints apply.

The initial stable identity could have a shape such as:

```rust
struct ExecutionCaseId {
    operation: architecture::OperationId,
    sponsor: SponsorCase,
    representations:
        BTreeMap<architecture::ObjectId, realization::RepresentationMode>,
}
```

with:

```rust
enum SponsorCase {
    Absent,
    Present,
}
```

No digest is needed.

## 4.2 Derive cases from typed semantics

Cases must derive from:

- the operation’s relation declarations;
- the selected proof plan;
- representation assignments;
- `RequirementActivation`;
- allowed optional sponsor flow.

Do not derive cases from:

- operation-name string matching;
- planning tables;
- target assumptions;
- test fixtures;
- target transaction positions.

## 4.3 Representation belongs to the case but is fixed by the plan

For a given proof-plan candidate, the representation is already selected.

Guide 5 should not cross that candidate with every other representation again.

Instead:

```text
proof plan A:
    live receipt = Explicit
    cases:
        sponsor absent
        sponsor present

proof plan B:
    live receipt = PrivateCommitted
    cases:
        sponsor absent
        sponsor present
```

Across the complete feasible plan set, both representation families are
covered.

This avoids creating an execution case that contradicts its proof plan.

## 4.4 Pilot case census

### Compact ASH

The complete initial case census should be:

| Representation | Sponsor |
|---|---|
| explicit | absent |
| explicit | present |
| public committed | absent |
| public committed | present |

No private-committed ASH case is allowed.

### Live transfer

The complete initial case census should be:

| Representation | Sponsor |
|---|---|
| explicit | absent |
| explicit | present |
| private committed | absent |
| private committed | present |

No public-committed live-transfer case is currently declared.

## 4.5 Do not over-expand cases

Guide 5 should not create separate placement cases for every:

- input count;
- output count;
- sponsor denomination;
- sponsor change value;
- owner identity;
- receipt denomination;
- object ordering.

Those values do not change the abstract placement for the pilots.

For example, “sponsor present with no change” and “sponsor present with
change” can remain one placement case if the same global carrier and optional
family requirements apply. P2-011 coverage may later require separate vectors
for the two shapes.

---

# 5. Relations need a typed discharge boundary

Guide 5 should define how each relation is discharged before assigning runtime
carriers.

A useful model is:

```rust
enum DischargeBoundary {
    CompilerStatic,
    BackendStructural,
    RuntimeCarrier,
    ExternalEvidence,
}
```

A relation may produce more than one downstream requirement. Therefore this
should not necessarily be stored as exactly one enum value per relation.

For example:

```rust
struct RelationCasePlan {
    relation: realization::RelationId,
    case: ExecutionCaseId,
    activity: RelationActivity,
    compiler_requirements: Vec<CompilerStaticRequirement>,
    structural_requirements: Vec<BackendStructuralRequirement>,
    runtime_requirements: Vec<RuntimePlacementRequirement>,
    external_evidence:
        BTreeSet<realization::ExternalEvidenceRequirement>,
}
```

This avoids forcing a hybrid relation into one category.

## 5.1 Compiler-static relations

These include properties the compiler can validate directly from typed input:

- allowed representation selection;
- constructibility availability;
- lifecycle-path declaration;
- proof-plan consistency;
- scope and relation census.

Compiler-static does not mean target-verified.

## 5.2 Backend-structural requirements

Some relations produce requirements on the future emitted bundle or ABI without
being ordinary runtime predicates.

Examples:

- selected representation must be encoded and authenticated;
- permissionless operation path must contain no owner/operator secret gate;
- required lifecycle exit must remain represented;
- operation family must have a canonical global coordinator where required;
- an optional carrier must not be the only carrier of an unconditional
  relation.

These are backend obligations, not completed target evidence.

## 5.3 Runtime-carried relations

These require an active semantic carrier in a concrete operation:

- object recognition;
- family cardinality;
- family closure;
- amount conservation;
- owner authorization;
- canonical partition;
- open-flow policy;
- sponsor isolation;
- root policy;
- projection policy.

For the pilots, roots are forbidden, but the root-policy relation still
requires the operation’s global relation to establish the absence of root
effects.

## 5.4 External-evidence relations

Substrate conservation belongs here.

It requires:

- no script-level semantic carrier merely because a relation exists;
- the abstract capability established by T6;
- an unresolved typed evidence requirement;
- later target/deployment evidence.

It must not be accidentally placed on a protocol carrier and treated as
complete.

---

# 6. Abstract carrier roles

## 6.1 Carrier roles must remain target-independent

A carrier is an abstract semantic enforcement role, not a transaction index.

Suitable roles include:

```rust
enum CarrierRole {
    EveryInputFamilyMember {
        object: architecture::ObjectId,
    },

    InputFamilyCoordinator {
        object: architecture::ObjectId,
    },

    OperationGlobal {
        operation: architecture::OperationId,
    },

    BackendStructural {
        operation: architecture::OperationId,
    },

    ExternalEvidence {
        requirement:
            realization::ExternalEvidenceRequirement,
    },
}
```

The exact names may differ.

The critical properties are:

- no `NodeIndex`;
- no input ordinal;
- no output ordinal;
- no tapleaf;
- no stack position;
- no witness item position;
- no target program ID;
- no transaction slot.

## 6.2 Avoid a magical coordinator

A generic `OperationGlobal` carrier should not appear from nowhere.

For runtime target operations, the compiler should derive possible coordinator
anchors from input families guaranteed to be present.

For the current pilots:

```text
compact ASH:
    ASH input minimum = 2
    → ASH input family is always present
    → eligible coordinator anchor

live transfer:
    live receipt input minimum = 1
    → live-receipt input family is always present
    → eligible coordinator anchor
```

An optional sponsor family must not be the sole carrier of an unconditional
relation.

## 6.3 Quantified local carriers

`EveryInputFamilyMember` is a quantified carrier role.

It does not enumerate a runtime number of members. It means:

```text
for every authenticated member of this bounded family,
the corresponding local obligation must execute
```

This is appropriate for:

- live-receipt recognition;
- live-receipt owner authorization;
- local class checks.

The family census and range authentication remain global requirements.

## 6.4 Output objects are not automatically executable carriers

Compiler core must not assume an output program executes while the output is
created.

Output-family recognition and closure should normally be assigned to a global
or input-anchored coordinator requirement.

A future target with whole-transaction semantics might realize that role
differently. The compiler should state the obligation, not assume tapscript
execution mechanics.

---

# 7. Carrier eligibility and source availability

A carrier is eligible only when every active source requirement can be supplied
there or made available by an explicit layout requirement.

Conceptually:

\[
\operatorname{Sources}(r,c)\subseteq\operatorname{Intrinsic}(p,c)\cup\operatorname{LayoutProvided}(p,c)
\]

where:

- \(r\) is a relation;
- \(c\) is an execution case;
- \(p\) is a carrier.

## 7.1 Intrinsic availability

Examples:

- a local live-receipt member has its own object facts;
- a local live-receipt member has its owner-authorization role;
- public architecture bounds are globally available;
- the selected representation is compiler-known;
- external evidence is available only at the external-evidence boundary.

## 7.2 Layout-provided availability

A future layout may need to make these facts available to a global carrier:

- authenticated family count;
- complete input-family census;
- complete output-family census;
- member asset/object facts;
- family amount totals;
- canonical partition;
- open-flow partition;
- projection set;
- root-effect set;
- owner-witness coverage;
- selected representation facts.

Guide 5 should record these as requirements. It must not claim they are already
implemented.

## 7.3 Private availability restrictions

Availability classes remain load-bearing:

```text
Public:
    may be supplied to a global carrier

InputOwners { object }:
    must remain tied to the authorized input family or an explicitly valid
    all-owner authorization carrier

SponsorLocal:
    may appear only in the sponsor-present case and only in the optional
    sponsor subtree/region

Operator:
    unavailable in current pilots

RefundKey:
    unavailable in current pilots
```

A sponsor-local fact must never leak into the protocol coordinator’s semantic
fact set as an exact sponsor value.

---

# 8. Layout requirements

## 8.1 Placement and layout are different

Guide 5 must keep this distinction explicit:

```text
placement:
    which carrier role is responsible for a relation?

layout requirement:
    what authenticated transaction structure and fact routing must exist
    so that carrier can discharge it?
```

A relation having a carrier does not prove the carrier can see its operands.

A layout requirement existing does not prove a backend implemented it.

## 8.2 Layout requirements should cite semantic owners

Avoid retyping semantic minima, maxima, or formulas into compiler-owned fields
when the relation ID already owns them.

Prefer:

```rust
enum LayoutRequirement {
    AuthenticateFamilyCensus {
        relation: realization::RelationId,
        side: realization::TransactionSide,
        object: architecture::ObjectId,
    },

    CompleteAndDisjointFamilies {
        relation: realization::RelationId,
        side: realization::TransactionSide,
    },

    CanonicalCoordinator {
        operation: architecture::OperationId,
        anchor: CarrierRole,
    },

    MakeSourceAvailable {
        relation: realization::RelationId,
        case: ExecutionCaseId,
        carrier: CarrierRole,
        source: SourceRequirement,
    },

    IsolateSponsorRegion {
        relation: realization::RelationId,
        case: ExecutionCaseId,
    },

    EnforceRepresentation {
        relation: realization::RelationId,
        object: architecture::ObjectId,
        representation: realization::RepresentationMode,
    },

    SecretFreeOperationPath {
        relation: realization::RelationId,
        case: ExecutionCaseId,
    },
}
```

The relation remains the semantic owner. The layout requirement says what the
backend must expose.

## 8.3 No concrete ranges yet

Guide 5 may require:

```text
authenticated bounded family
complete family membership
canonical coordinator
disjoint protocol and sponsor regions
```

It should not assign:

```text
inputs 0..N
output 3
lowest transaction index
witness item 7
tapleaf X
stack slot Y
```

Those belong below the compiler boundary.

## 8.4 Sponsor layout requirements

For sponsorless cases:

- the sponsor input/output families are empty;
- sponsor-specific witness requirements are inactive;
- sponsor-member predicates may be vacuous;
- the global relation still proves no sponsor region exists.

For sponsored cases:

- one optional sponsor region exists;
- every member is claimed exactly once;
- sponsor/protocol references are disjoint;
- input owners authorize sponsor inputs;
- at-most-one envelope holds;
- individual sponsor amounts remain erased;
- whole-transaction conservation remains external evidence.

---

# 9. Relation classification for the two pilots

Guide 5 should contain an explicit tested classification matrix.

## 9.1 Compact ASH

| Relation family | Conceptual discharge |
|---|---|
| ASH input recognition | every ASH input member or complete coordinator proof |
| ASH output recognition | global coordinator |
| ASH cardinality | global coordinator |
| allowed input/output families | global coordinator |
| ownerless `U` conservation | global coordinator |
| canonical delta policy | global coordinator |
| permissionless authorization | backend-structural secret-free path |
| constructibility | compiler-static |
| representation | compiler-static selection plus backend-structural encoding |
| compact/clear lifecycle | compiler-static requirement, later bundle/ABI structural evidence |
| sponsor recognition/cardinality | sponsor-region/global coordinator when present; vacuous member checks when absent |
| sponsor isolation/multiplicity | global coordinator |
| open-flow policy | global coordinator |
| root policy | global coordinator proving no root effects |
| projection policy | global coordinator proving transition certificate only |
| substrate conservation | external evidence |

The mandatory coordinator anchor is the ASH input family.

## 9.2 Live transfer

| Relation family | Conceptual discharge |
|---|---|
| live input recognition | every live-receipt input member or complete coordinator proof |
| owner authorization | every consumed live-receipt member |
| live output recognition | global coordinator |
| input/output cardinality | global coordinator |
| input/output family closure | global coordinator |
| aggregate `U` conservation | global coordinator |
| canonical delta policy | global coordinator |
| constructibility | compiler-static |
| representation | compiler-static selection plus backend-structural encoding |
| transfer/burn/redeem lifecycle | compiler-static requirement, later bundle/ABI structural evidence |
| sponsor recognition/cardinality | sponsor-region/global coordinator when present |
| sponsor isolation/multiplicity | global coordinator |
| open-flow policy | global coordinator |
| root policy | global coordinator proving no root effects |
| projection policy | global coordinator proving transition certificate only |
| substrate conservation | external evidence |

The mandatory coordinator anchor is the live-receipt input family.

---

# 10. Exact placement search

## 10.1 What the search chooses

For every proof-plan candidate and execution case, placement chooses eligible
carrier roles for runtime relations.

It does not choose:

- proof alternatives;
- representations;
- target programs;
- concrete indexes;
- one final deployment plan.

Those decisions are already made earlier or belong later.

## 10.2 Hard constraints

For every relation \(r\) and case \(c\):

\[
\operatorname{Active}(r,c)\Rightarrow\exists p:\operatorname{Assigned}(r,c,p)\land\operatorname{Eligible}(r,c,p)
\]

Additional constraints:

- every-member relations use a quantified every-member carrier;
- family-global relations require complete family census;
- transaction-global relations require a complete global carrier;
- optional sponsor carriers cannot uniquely carry unconditional relations;
- external-evidence relations have no runtime carrier;
- compiler-static relations have no runtime carrier;
- every selected carrier receives every required source or an explicit layout
  requirement;
- deliberately duplicated enforcement is permitted only when typed as such;
- no relation disappears from the relation/case census.

## 10.3 Do not generate arbitrary duplicate supersets

If every feasible one-carrier placement is retained along with every arbitrary
superset, the candidate set grows exponentially without adding semantic value.

Guide 5 should retain either:

- exactly one carrier where multiplicity is `ExactlyOne`;
- every member where multiplicity is `EveryMember`;
- inclusion-minimal carrier sets where multiple carriers are allowed;
- explicitly required duplicate sets where policy requires duplication.

Do not treat “more carriers” as automatically better.

## 10.4 No cost optimization yet

Guide 5 should not introduce a resource objective. There is no target resource
model yet.

The result is:

```text
complete feasible target-independent placement set
```

not:

```text
cheapest placement
```

If several feasible placements remain, retain them in canonical order.

## 10.5 Complexity limits

Add explicit limits such as:

```rust
struct PlacementSearchLimits {
    maximum_states: NonZeroU64,
    maximum_candidates: NonZeroU64,
}
```

These may initially join `AnalysisPolicy` if they have a real consumer.

Exhaustion returns a typed error and no partial result:

```text
PlacementSearchStateLimitExceeded
PlacementCandidateLimitExceeded
```

Do not silently select the first placement seen.

---

# 11. Stable projection

Guide 5’s stable projection must contain only typed semantic values:

```text
proof-plan typed value
execution-case IDs
relation IDs
carrier roles
relation/case assignments
layout requirements
external evidence requirements
search limits where semantically configuration-relevant
```

It must exclude:

- Petgraph node and edge indices;
- DFS/BFS order;
- search-state number;
- candidate vector index;
- elapsed time;
- temporary paths;
- thread scheduling;
- source line numbers;
- diagnostics;
- target bytes.

Repeated analysis and declaration permutations must produce equal projections.

No public hash should be added.

---

# 12. Direct Petgraph use

Under D007, graph-shaped placement state should use Petgraph directly.

A useful internal graph is a bipartite or multipartite graph among:

```text
relation-case requirement nodes
carrier-role nodes
source/layout requirement nodes
```

Typed edges may express:

```text
eligible carrier
requires source
requires layout
covers relation case
```

The compiler may maintain:

```text
stable typed key ↔ NodeIndex
```

as local metadata.

Do not introduce a generic wrapper such as:

```text
PlacementGraph
CanonicalPlacementGraph
```

if it merely wraps Petgraph storage. A domain-specific analysis struct
containing a concrete Petgraph graph and metadata is acceptable.

---

# 13. Independent placement oracle

The independent oracle is essential because the production placement search
will be another exact combinatorial algorithm.

For small synthetic inputs:

1. enumerate every allowed carrier subset or assignment;
2. evaluate hard constraints directly;
3. retain inclusion-minimal feasible placements;
4. sort by stable typed projection;
5. compare the entire feasible set with production.

The oracle should not call production:

- carrier eligibility;
- relation classification;
- case activation;
- layout-completeness helper.

It may share stable types but must restate the hard predicates independently.

Required oracle cases:

- one mandatory family and one coordinator;
- two eligible coordinators;
- no eligible carrier;
- optional sponsor carrier only;
- unconditional relation on optional carrier;
- member-local relation missing one member role;
- transaction-global relation assigned only locally;
- source unavailable at carrier;
- layout-provided source;
- external relation incorrectly assigned runtime carrier;
- static relation incorrectly assigned runtime carrier;
- deliberate duplicate enforcement;
- equal feasible placement permutations;
- state-limit exhaustion;
- candidate-limit exhaustion.

---

# 14. Recommended internal modules

A clean split would be:

```text
packages/compiler/src/case.rs
packages/compiler/src/placement.rs
packages/compiler/src/layout.rs
```

Possible ownership:

## `case.rs`

- `ExecutionCaseId`;
- case dimensions;
- case derivation from proof candidates;
- source-requirement activation;
- case-census validation.

## `placement.rs`

- discharge classification;
- carrier roles;
- carrier eligibility;
- exact placement search;
- placement candidates;
- stable projection;
- search report.

## `layout.rs`

- target-independent layout requirements;
- source-at-carrier requirements;
- family census and closure requirements;
- sponsor-region requirements;
- representation encoding requirements;
- layout-requirement census validation.

Add each new Rust source to:

```text
packages/compiler/meson.build
```

The internal analyses should remain crate-private until P2-012.

---

# 15. Error vocabulary

Guide 5 will likely need focused errors. Suitable classes include:

```rust
DuplicateExecutionCase {
    case: ExecutionCaseId,
}

ExecutionCaseCensusMismatch {
    missing: Vec<ExecutionCaseId>,
    unexpected: Vec<ExecutionCaseId>,
}

MissingRelationCasePlan {
    relation: realization::RelationId,
    case: ExecutionCaseId,
}

NoEligibleCarrier {
    relation: realization::RelationId,
    case: ExecutionCaseId,
}

GlobalRelationHasOnlyLocalCarrier {
    relation: realization::RelationId,
    case: ExecutionCaseId,
}

UnconditionalRelationOnOptionalCarrier {
    relation: realization::RelationId,
    case: ExecutionCaseId,
}

MissingCarrierSource {
    relation: realization::RelationId,
    case: ExecutionCaseId,
    carrier: CarrierRole,
    operand: OperandId,
}

MissingLayoutRequirement {
    relation: realization::RelationId,
    case: ExecutionCaseId,
}

PlacementCensusMismatch {
    missing: Vec<RelationCaseKey>,
    unexpected: Vec<RelationCaseKey>,
}

PlacementSearchStateLimitExceeded {
    maximum: u64,
}

PlacementCandidateLimitExceeded {
    maximum: u64,
}
```

The exact variants should reflect actual implementation branches. Do not add
speculative variants that no code path can produce.

---

# 16. Required tests for the real pilots

## 16.1 Case census

Assert exactly four semantic cases across feasible candidates for each pilot:

```text
compact ASH:
    explicit × sponsor absent/present
    public committed × sponsor absent/present

live transfer:
    explicit × sponsor absent/present
    private committed × sponsor absent/present
```

The representation is fixed by each proof plan, so individual candidates may
carry only two sponsor cases.

## 16.2 Relation-case census

Require exact equality:

```text
every in-scope relation
×
every applicable execution case
=
relation-case analysis census
```

Vacuous, compiler-static, backend-structural, runtime, and external
dispositions must all be represented explicitly.

## 16.3 Mandatory coordinator

Assert:

```text
compact ASH:
    ASH input family can anchor the global coordinator

live transfer:
    live-receipt input family can anchor the global coordinator
```

Assert that optional sponsor input cannot be the sole global coordinator.

## 16.4 Local owner authorization

For live transfer:

```text
owner authorization
    → every live-receipt input member
```

A placement on only one coordinator must not silently replace all-owner
authorization unless an explicit complete all-owner proof alternative exists
and is modeled.

## 16.5 Global conservation

For both pilots:

```text
amount conservation
    → complete global carrier
    → authenticated input/output family totals
```

A per-member-only assignment must fail.

## 16.6 External conservation

For both pilots:

```text
substrate conservation
    → external-evidence disposition
    → no runtime protocol carrier
    → WholeTransactionValueConservation capability retained
```

## 16.7 Sponsor erasure

No placement or layout requirement may name:

- sponsor input amount;
- sponsor output amount;
- sponsor positivity;
- public sponsor sum.

Sponsor requirements may name:

- family membership;
- owner authorization;
- exact reference membership;
- region disjointness;
- envelope multiplicity;
- external conservation evidence.

## 16.8 Determinism

Test:

- proof-plan candidate order permutations;
- relation declaration permutations;
- carrier declaration permutations;
- execution-case permutations;
- repeated analysis equality;
- stable diagnostic ordering.

---

# 17. What Guide 5 should not claim

At completion, the correct statement is:

> The compiler has derived a complete target-independent set of execution cases,
> carrier requirements, feasible abstract placements, and layout obligations for
> the two pilots.

It must not say:

- the relations are enforced by tapscript;
- a coordinator input index has been selected;
- an ABI exists;
- the target can expose every required fact;
- every carrier is reachable in a linked bundle;
- relation-indexed vectors pass;
- target resources fit;
- deployment evidence is complete;
- the compiler is complete.

Those claims belong to later phases.

---

# 18. Suggested Guide 5 sequence

A well-ordered implementation guide should proceed as follows.

## Tranche A — Execution cases

1. define stable case types;
2. derive proof-plan-specific cases;
3. filter source requirements by activation;
4. validate exact case census;
5. add pilot case tests.

## Tranche B — Relation discharge classification

1. define independent discharge axes;
2. classify every current relation variant exhaustively;
3. distinguish compiler-static, backend-structural, runtime, and external
   requirements;
4. add classification matrix tests.

## Tranche C — Carrier model

1. define stable abstract carrier roles;
2. derive mandatory input-family coordinator anchors;
3. derive local every-member roles;
4. reject optional-only carrier defects;
5. add carrier-eligibility tests.

## Tranche D — Layout requirements

1. derive family-census requirements;
2. derive source-at-carrier requirements;
3. derive complete/disjoint family requirements;
4. derive sponsor-region requirements;
5. derive representation and secret-free structural requirements;
6. validate exact requirement census.

## Tranche E — Exact placement

1. enumerate feasible placements;
2. retain only policy-permitted minimal/required carrier sets;
3. add explicit search limits;
4. return no partial result on exhaustion;
5. canonicalize candidates.

## Tranche F — Independent oracle

1. enumerate small carrier assignments independently;
2. compare complete feasible sets;
3. cover adversarial optional/global/local cases;
4. test insertion permutations.

## Tranche G — Pilot integration

1. run compact ASH through cases, placement, and layout;
2. run live transfer through cases, placement, and layout;
3. assert exact relation/case census;
4. assert repeated equality;
5. keep all analysis crate-private.

## Tranche H — Documentation and gate

1. update compiler README and Phase-2 card;
2. mark C1-009 and P2-010 complete only after verification;
3. make P2-011 the next active task;
4. run focused and full gates;
5. record skipped/deferred lanes honestly;
6. require a clean tree.

---

# 19. Recommended exit criteria

Guide 5 should be accepted only when:

- Guide 4 is fully complete;
- execution cases are typed and deterministic;
- cases derive from selected proof plans and typed activations;
- relation discharge boundaries are explicit;
- conditionality and multiplicity are separate from locality;
- every active runtime relation has an eligible abstract carrier;
- member-local obligations retain every-member multiplicity;
- transaction-global relations cannot hide on local-only carriers;
- optional sponsor carriers cannot carry unconditional relations alone;
- external evidence is not assigned a runtime carrier;
- compiler-static relations are not misreported as target execution;
- every carrier has complete source or layout requirements;
- layout requirements remain target-independent;
- sponsor amounts remain erased;
- exact placement search returns the complete feasible set;
- complexity exhaustion returns no partial result;
- the independent oracle agrees on every generated small instance;
- both pilots have exact case and relation-case censuses;
- stable projections contain no local graph handle;
- no target position, opcode, stack item, tapleaf, or ABI field enters compiler
  core;
- no new digest is minted;
- no new dependency is added without separate review;
- workspace and repository gates pass;
- the final tree is clean.

---

## Bottom line

Guide 5 should not be “put relations on future scripts.” It should establish a
typed, exact intermediate contract:

```text
for each feasible proof plan,
for each semantic execution case,
for each relation:

    what kind of discharge is required?
    which abstract carrier roles are eligible?
    what facts must be available there?
    what layout constraints must a backend satisfy?
```

Once that result is correct, Guide 6 can safely implement P2-011’s
relation-indexed coverage requirements. Only after both placement and coverage
exist should P2-012 expose the complete pilot analyzed program.
