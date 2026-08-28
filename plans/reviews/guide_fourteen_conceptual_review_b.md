# Reviewer B — Linking the mutable STATE cycle

## §1. Question restated

This review asks what kind of recursion Guide 14 actually creates when a STATE output commits to a static operation subtree whose own announcement leaf authenticates that output and constructs its successor. The central issue is whether the guide turns that apparent self-reference into a standard, finite, authenticated state transition, or merely gives a cyclic reference graph a project-specific label.

I evaluate four questions under the owner's doctrine:

1. At the top of the pipeline, does the design present one genuinely mutable canonical STATE to typed semantics?
2. At the back end, does it realize each mutation as a consume-predecessor/reconstruct-successor UTXO transition using established linking and graph guarantees?
3. Are branch orientation, tweak handling, nonce selection, declaration-order independence, and constructor continuity deterministic and authenticated, rather than conventions followed only by the host builder?
4. Across successive edges and chain reorganizations, is the recursion well-founded and is there one chain-relative current STATE rather than several constructor instances that can all claim canonicality?

The relevant comparison points are recursive bindings and `letrec`, compiler and object-file symbol resolution, SCC condensation, feedback-edge cuts, quines and recursive covenants, content-addressed Merkle structures, canonical-representative search, event-sourced state machines, and chain checkpoint/finality discipline.

## §2. The guide's actual design as I read it

At the semantic level, Guide 14 defines an abstract state-machine transition. A typed `StateMetadata` value changes only from unannounced maturity to an announced cycle; all other fields are preserved (§1.1, §1.4, §6.1–§6.5). The request does not choose the predecessor, successor metadata, program, static subtree, internal key, or representation nonce (§2.3). This is a good expression of mutable canonical state at the top of the pipeline: the operation is described as a mutation of one logical STATE, not as the creation of an unrelated output.

At the target level, the mutation is linearized as a UTXO edge. The transaction consumes exactly one current STATE outpoint and creates exactly one successor at output 0 (§1.3, §2.2–§2.6, §12.1–§12.2). The predecessor and successor programs are both instances of the same constructor recipe (§1.7): a dynamic, always-aborting metadata leaf sits beside one exact static operation subtree under a fixed public internal key (§5.2, §9.1–§9.6). Metadata changes; the static subtree does not.

The apparent recursive equation is roughly:

```text
static root C = Merkle root of operation leaves including announce(C)
STATE program Q(M, C) = tweak(internal key, branch(metadata leaf M, C))
```

The accepted prototype does not solve this as a cryptographic fixed point. Instead, the executing leaf receives one static-root value, authenticates it by reconstructing the actual consumed program from predecessor metadata, retains that authenticated value, and reuses it to reconstruct the successor (§9.5–§9.7, §10.3, §10.7). The actual consumed program comes from target introspection, not from a link-time literal. This is the essential knot-breaking move. Repeated hashing until bytes stabilize is expressly prohibited (§9.7, §16.7, §22).

The linker is planned as follows:

- Typed symbol keys name deployment constants, schema facts, the static subtree, the internal key, and other consumers (§11.1).
- Pass one collects, type-checks, rejects duplicate definitions, and normalizes them by stable typed key (§11.2).
- Pass two resolves references, freezes the graph, computes SCCs, and applies a cycle policy (§11.2).
- Relocations are located structurally, carry a typed semantic consumer and resource effect, and must apply exactly once (§11.3).
- Tree assembly rejects duplicate leaves, uses checked costs and stable typed tie-breaks, and claims declaration-order independence (§11.4; Wave 6).

The guide is not fully consistent about the exact cycle criterion. Section 9.7 says any cyclic edge requires a named authenticated cut. Wave 6 and §21 instead require every cycle to have an authenticated strategy. Section 11.2 says only “apply cycle policy.” The inherited linker substrate is more specific: it currently accepts an SCC when at least one internal edge has a resolving class. That is sufficient for the one simple compact-ASH loop it was built around, but is not the standard criterion for a general SCC containing several independent directed cycles.

Constructibility is also split into several choices. The program hashes the metadata leaf on a fixed side and admits only instances whose metadata-leaf hash is no greater than the static root (§5.3). A public representation nonce is searched from zero upward until both branch-side admissibility and target tweak validity hold (§5.3–§5.4, §12.5). The internal key is a fixed public NUMS point under an explicit discrete-log residual (§5.5). The guide repeatedly calls the first admissible nonce canonical and requires a later admissible nonce to reject (§1.5, §6.3, §16.6, §21).

Temporally, the guide records an ordered outpoint edge sequence and advances the current cursor only after validating each edge (§14.6, §17.1). Checkpoint evidence binds network, genesis, block hash and height, transaction, predecessor and successor outpoints, target contract, bundle, and ABI; a reorg stales the observation (§17.2). Constructor migration is forbidden, so the static subtree is an invariant across the edge (§17.3).

## §3. Mapping to standard concepts and algorithms

| Guide mechanism | Established concept | Assessment |
|---|---|---|
| Typed mutable STATE plus `announce_maturity` | Abstract data type and deterministic state-transition system | Faithful at the semantic level. The transition is expressed over typed state and the request supplies an operation argument, not a replacement state (§1.4, §6.5). |
| Consume one outpoint, create one outpoint | Linear resource semantics; compare-and-swap on a versioned cell | Faithful on one canonical chain. Spending the exact predecessor is the compare condition; output 0 is the successor. Consensus double-spend prevention serializes competing transitions on that chain (§1.3, §12.1–§12.2). |
| Typed symbol census | Typed symbol table and static name resolution | Faithful and stronger than string substitution. Type, origin, duplicate rejection, and stable symbol keys are the normal compiler obligations (§11.1–§11.2). |
| Two-pass resolution | Assembler/compiler forward-reference resolution | Faithful for acyclic references: collect all definitions, then resolve uses against the closed environment (§11.2). It does not by itself solve recursive values. |
| SCC computation | Tarjan/Kosaraju mutual-recursion classification and condensation DAG | Correct as diagnosis, not yet sufficient as acceptance. SCCs identify knots; a separate proof must show that authenticated runtime edges form a feedback edge set. |
| Static-root witness authenticated against the actual input | Open recursion with a spend-time environment; guarded recursive binding; proof-carrying indirection | This is the real resolution mechanism. It is analogous to elaborating `letrec` through an explicit cell or environment, except the “cell” is the actual consumed UTXO program and the supplied root is accepted only after reconstructive authentication (§9.5–§9.7). |
| Input-program introspection | Quine/self-description mechanism; recursive covenant introspection | Faithful in spirit. A quine obtains its own description through a standard self-reference construction; this covenant obtains its current identity from the target rather than embedding an infinitely regressing literal. It is not a hash fixed point. |
| Reuse one authenticated root for the successor | Loop invariant and inductive preservation | Strong and simple. The program retains one authenticated value instead of comparing two independently supplied roots (§9.5–§9.6, §10.7). |
| Dynamic metadata leaf plus static code template | Commitment to a template/recipe rather than to a recursively embedded identity | This is the covenant literature's standard escape from content-addressed cycles. Merkle structures are naturally DAGs; an identity hash cannot generally contain itself. The guide's sound form commits to a recipe and authenticates the current instantiation at spend time. |
| Structured relocations | Object-file relocation over typed intermediate representation | Mostly faithful. Structural sites, expected types, exact multiplicity, and resource effects are good (§11.3). The guide omits several standard safety clauses stated in its own linker research: validate all sites first, substitute from pristine input, apply simultaneously, and reparse the result. |
| Fixed branch side and nonce grinding | Canonical representative as the least member satisfying a predicate; bounded exhaustive search | This is not a fixed point. It is a deterministic search over a well-ordered finite domain. It terminates at the bound and has a unique result only if the admissible set is nonempty and leastness is enforced at the relevant trust boundary (§5.3–§5.4). |
| Fixed NUMS internal key | Transparent setup / nothing-up-my-sleeve parameter | Faithful and honestly scoped. Deterministic derivation and absence of a key-generation step are evidence; absence of any scalar remains a discrete-log assumption (§5.5). It anchors the constructor but does not break the reference cycle. |
| Ordered root-edge report | Event sourcing and inductive history validation | Faithful for the observed prefix. Final cursor equality is correctly rejected as a substitute for validating each event (§14.6, §17.1). |
| Block checkpoint and reorg staleness | Versioned snapshot with optimistic invalidation | Sound as an evidence statement, but incomplete as operational canonical-state management. A stale snapshot must also fence construction/submission and trigger rollback or replay (§17.2). |

Fixed-point combinators, the recursion theorem, and quines explain how self-reference can be represented without textual regress, but they do not make a cryptographic hash equation soluble. Kleene iteration or least-fixed-point theorems require order-theoretic structure such as monotonicity over a complete domain; a cryptographic hash has none. The guide is therefore correct to forbid iterative hash stabilization. Its actual standard form is runtime indirection plus authenticated reconstruction, not fixed-point search.

## §4. Findings, ranked

### 1. CONCEPT-BLOCKER — The SCC acceptance rule is not stated as a feedback-edge proof

**Sections implicated:** §9.7, §11.2, Wave 6, §21 “Constructor,” §22.

An SCC may contain many directed cycles. Marking one edge in the SCC as introspected or externally authenticated does not prove that all the other cycles have been broken. The inherited linker currently implements exactly that weaker rule: one resolving edge anywhere in the component accepts the component. This happens to work for a single simple self-commitment loop, but it is not the standard graph-theoretic guarantee Guide 14 claims for general constructor SCCs.

The correct criterion is: classify and validate each proposed runtime cut edge; remove those edges from the link-time dependency graph; then require the residual graph to be acyclic. Equivalently, the authenticated cuts must form a feedback edge set intersecting every directed cycle. The remaining graph can then be linked in a canonical topological order. This test is linear after classification and is simpler than reasoning about every enumerated cycle.

**Recommendation:** Make this residual-acyclicity test the Wave-6 acceptance rule and add an SCC with two edge-disjoint cycles as the decisive negative. Do not inherit the current “one cut per SCC” rule.

### 2. CONCEPT-BLOCKER — The recursive symbol lacks an explicit binding-time type

**Sections implicated:** §1.7–§1.8, §9.5–§9.7, §11.1–§11.3, §12.6.

`StateStaticSubtree` appears in the ordinary symbol census, while the sound constructor strategy requires it to be a spend-time authenticated parameter. If the linker treats that symbol like a normal definition and relocates its root bytes into a descendant operation leaf, the hash self-cycle returns and no two-pass linker can solve it. A forward declaration or `letrec` allocation helps only when the language has indirection; content-addressed immutable bytes do not.

The guide describes the right runtime behavior but does not make it unrepresentable to choose the wrong binding mode. “Every mandatory reference resolves exactly once” is also misleading for this symbol: it should not resolve to a link-time value at all. It should resolve to a typed strategy whose value is obtained and authenticated at spend time.

**Recommendation:** Type references by binding time, for example `LinkTimeDefinition`, `SpendTimeIntrospection`, and `WitnessedThenAuthenticated`. Require a non-occurrence proof that the static-root value is never serialized as a literal into any leaf beneath that root. Record the exact introspection instruction, witness role, reconstruction equation, and equality that authenticates it. Only after that cut should ordinary symbol resolution and relocation run.

### 3. CONCEPT-BLOCKER — “First admissible nonce” is selected by the host but not shown to be authenticated by the target

**Sections implicated:** §1.5, §5.3–§5.4, §6.3, §9.6, §10.7, §12.5, §16.6–§16.7, §21 “Constructor.”

The guide requires the first admissible nonce and says a later admissible nonce must reject. Its concrete mechanism only establishes that the witnessed nonce produces a fixed-side branch and a valid tweaked successor matching output 0. A later nonce that also satisfies those predicates produces a different but internally consistent successor program. Reconstructing that program does not prove that every smaller nonce failed.

The host API's refusal to accept a caller-supplied nonce is not a back-end protocol guarantee. The operator signature binds the chosen bytes, but authorization is not canonicalization. The accepted prototype substrate confirms the distinction: the target program checks that the chosen nonce leads to the actual output; the ordered search lives in creator-side construction.

This leaves more than one valid constructor representation for the same semantic successor, contrary to the unique-canonical-state claim and the required later-nonce negative. It also gives reorg competitors a way to carry distinct constructor bytes for the same semantic transition.

**Recommendation:** Either prove leastness in the target program by checking every earlier candidate within a small, fixed, resource-proven bound; redesign the commitment so no grinding choice exists; or explicitly demote leastness to an operator policy and stop claiming target-enforced canonical bytes. The last option is a substantive weakening of the stated doctrine, since it retires the claim the target-enforcement argument rests on.

### 4. DESIGN-RISK — The nonce policy guarantees bounded termination, not constructor totality

**Sections implicated:** §5.4, §6.5, §9.8, §12.5, §18.1, §21 “Constructor.”

The standard name for the algorithm is “least satisfying element by bounded exhaustive search.” Its theorem is conditional:

- it terminates after at most the fixed number of attempts;
- if at least one admissible nonce exists, it returns a unique least one;
- otherwise it returns exhaustion.

There is no unique fixed-point theorem here. Expected success follows only from modeling the hash/tweak predicates probabilistically. The guide is commendably explicit about exhaustion, but still uses “tweak totality” and describes the semantic transition as total while permitting a semantically valid successor to be unconstructible at the back end. That is precisely where the two levels of the owner's doctrine can diverge.

**Recommendation:** Fix the nonce width and maximum attempts as constructor-policy inputs; state the exact attempt count convention; quantify the residual under the adopted random-oracle/group model; and settle whether backend exhaustion is a permitted typed state-transition refusal. If all valid semantic mutations must be realizable, the present policy does not meet that requirement.

### 5. DESIGN-RISK — Two-pass linking does not by itself force declaration-order-independent bytes

**Sections implicated:** §7.5, §11.2–§11.4, Wave 6 exit, §21 “Linking and ABI.”

Sorting definitions is necessary but insufficient. Byte independence also requires canonical ordering of reference edges, SCC members, SCC identities, condensation-DAG topological ties, relocation sites, tree leaves, equal-cost tree candidates, and final serialization. A deterministic implementation can still deterministically preserve declaration order.

Section 11.3 also does not say whether several relocations are applied simultaneously from pristine structured input. Sequential substitution can become order-sensitive when a replacement changes a later consumer's structure or resource calculation.

**Recommendation:** Define one canonical normal form for the complete linker input. Compute stable graph and SCC keys, use a stable-key priority queue for topological ties, rebuild structured programs as a pure function of the closed assignment, reparse them, and assemble the tree only from canonical leaf records. State and test permutation invariance over definitions, references, relocations, leaves, and equal-cost ties. This is the standard confluence property behind the guide's byte claim.

### 6. DESIGN-RISK — Checkpoint binding detects a reorg but does not yet manage mutable canonical state through one

**Sections implicated:** §1.3, §12.4, §17.1–§17.3, Wave 10, §23.

The UTXO model provides excellent local linearity: on one selected chain, two transitions cannot both consume the same predecessor. Outpoints also distinguish state instances even when their constructor programs are byte-equal. Section 17 correctly makes observations chain-prefix-relative and stale after a reorg.

However, staleness is only detection. During a fork, two valid successors of the same predecessor can be live in different views. The guide does not define confirmation/finality policy, how a cached `current-STATE view` is fenced at submission time, or how typed top-level STATE rolls back and replays when the checkpoint disappears. Without those rules, “current canonical STATE” is an assertion of the selected view, not a complete temporal guarantee.

**Recommendation:** Treat the current-root view as a versioned compare-and-swap token bound to an exact chain checkpoint. Revalidate it immediately before signing/submission, define confirmation/finality status, invalidate all derived candidates and reports on checkpoint loss, and specify deterministic rollback/replay to the common ancestor. State explicitly that uniqueness is chain-relative before finality.

### 7. OBSERVATION — With the runtime cut made explicit, the constructor is coinductive but each execution is finite

**Sections implicated:** §1.7, §9.5–§9.7, §17.1, §17.3.

The linked recipe describes an unbounded stream of possible STATE successors, but an individual transaction checks only its predecessor and one successor. No edge expands the full future constructor history, so there is no operational regress. The same static subtree is a loop invariant, and root-history validation proves its preservation inductively, one outpoint edge at a time.

This is the standard and desirable form for a mutable recursive covenant. It is stronger and simpler to describe as local transition preservation over a linear UTXO history than as finding a global fixed point. Guide 14 proves only the announcement edge; lifecycle-wide coinduction remains unavailable until later STATE operations and migration rules exist (§2.7, §17.3, §26).

**Recommendation:** Adopt “open recursive constructor plus inductive edge invariant” as the normative explanation and reserve “fixed point” for the impossible literal-hash equation that the design rejects.

### 8. OBSERVATION — The safest current-target strategy is standard, but one architectural alternative is strictly simpler

**Sections implicated:** §5.1–§5.5, §9.2–§9.7, §11, §17.3.

Within the current tree-shaped STATE design, spend-time introspection plus one authenticated static-root witness is the standard solution. It is preferable to quine-like literal reconstruction and far preferable to hash iteration. The fixed NUMS key and fixed branch side are reasonable target-specific anchors once their assumptions are carried explicitly.

A strictly simpler linker would separate code identity from mutable data: keep one fixed recursive covenant program and place the mutable metadata commitment in a separately authenticated field or companion output. That is the classic indirection/template-commitment design. It removes the constructor hash cycle, static-root relocation, branch-side grinding, and tweak-totality search. Its cost is architectural: the protocol must define pairing, uniqueness, and atomic succession of the code and metadata objects. The guide already recognizes this tradeoff in the constructor research but does not admit it without an upstream decision.

Other alternatives are not strictly better here:

- A native target primitive that computes or verifies TapBranch ordering would remove nonce grinding, but the reviewed target does not provide it.
- A normalized template hash that excludes the self slot is safe if domain-separated and if runtime checks bind the slot, but it is essentially the explicit runtime-cut design under a clearer commitment format.
- Embedding metadata in every operation leaf increases recursive coupling and tree churn.
- Cryptographic fixed-point iteration has no standard correctness or termination argument and should remain prohibited.

## §5. Open questions for the owner

1. Must canonical representation be enforced by target execution against a malicious authorized operator, or is “the approved host builder selected it” considered sufficient? The current text claims the stronger property.
2. Is `StateStaticSubtree` intended to be a link-time definition, a spend-time introspected value, or a witnessed value authenticated through reconstruction? The type should permit exactly one answer.
3. Is a semantically valid STATE mutation allowed to return `RepresentationSearchExhausted`, or must the back end realize every valid typed transition?
4. Should Wave 0 reopen the inherited linker cycle rule, given that “one resolving edge per SCC” is not a feedback-edge guarantee?
5. What is the canonicality horizon for STATE: mempool tip, best block, a confirmation depth, or an owner-selected finalized checkpoint?
6. On reorg, which component owns rollback and deterministic replay of typed STATE, and what fences already-finalized signing requests derived from the stale root?
7. Is the architectural cost of a separate authenticated metadata commitment acceptable if target enforcement of least-nonce canonicality proves too expensive?
8. Before later STATE operations arrive, should Guide 14 claim only one-edge constructor preservation rather than a closed mutable cycle?

## §6. Wall-time and verification record

All substantive shell reads were wrapped with `/usr/bin/time`. Times are GNU `time` elapsed seconds as printed by `%e`; `0.00` means the cached read completed below one hundredth of a second. Every listed command exited 0. The broad discovery search was truncated by the tool display, so every source used substantively was then read directly in a bounded command.

| Step | Command / purpose | Wall time (s) |
|---:|---|---:|
| 1 | `git status --short --branch` — entry cleanliness | 0.01 |
| 2 | `git rev-parse HEAD` — revision pin | 0.00 |
| 3 | `rg` guide headings — section map | 0.00 |
| 4 | `sed -n 1,700p` Guide 14 — mission through scope | 0.00 |
| 5 | `sed -n 698,1405p` Guide 14 — substrate, decisions, semantics, compiler | 0.00 |
| 6 | `sed -n 1406,2131p` Guide 14 — constructor, linker, ABI, signing | 0.00 |
| 7 | `rg` constructor/linker concepts across `plans/` and `packages/` — substrate discovery | 0.03 |
| 8 | `sed -n 1,540p` state-constructor research — accepted prototype | 0.00 |
| 9 | `sed -n 1,580p` linker-algorithms research — intended guarantees | 0.00 |
| 10 | `sed -n 1,360p` linker package contract — inherited cycle rule | 0.00 |
| 11 | `sed -n 1,260p` constructor canonicalization source — branch-side search | 0.00 |
| 12 | `sed -n 1,260p` constructor totality source — bounded retry semantics | 0.00 |
| 13 | `sed -n 2132,2832p` Guide 14 — evidence, vectors, history | 0.00 |
| 14 | `sed -n 2833,3720p` Guide 14 — waves, acceptance, exit, impact | 0.00 |
| 15 | `sed -n 3721,4162p` Guide 14 — interchange, report, handoff | 0.00 |
| 16 | `sed -n 1,460p` linker graph source — graph and SCC construction | 0.00 |
| 17 | `sed -n 460,760p` linker graph source — cycle policy | 0.00 |
| 18 | `rg` checkpoint/root-history substrate — temporal context discovery | 0.00 |
| 19 | `rg` retained-chain and construction-view substrate — state-view discovery | 0.00 |
| 20 | `sed -n 1,260p` transaction public view — outpoint canonicalization | 0.00 |
| 21 | `rg` nonce canonicality in constructor/tapscript substrate — enforcement search | 0.00 |
| 22 | `sed -n 990,1115p` tapscript schedule tests — successor nonce behavior | 0.00 |
| 23 | `sed -n 1400,1545p` Guide 10 — constructor equation and root witness | 0.00 |
| 24 | `sed -n 1545,1645p` Guide 10 — branch order and internal key | 0.00 |
| 25 | `sed -n 1645,1735p` Guide 10 — tweak-totality alternatives | 0.00 |
| 26 | `rg` linker graph tests — cycle test inventory | 0.00 |
| 27 | `sed -n 180,230p` linker graph tests — one-cut acceptance | 0.00 |
| 28 | `ls -ld` report directory — destination check | 0.00 |
| 29 | `test ! -e` report path — non-overwrite check | 0.00 |
| 30 | `wc -l` report — read-back existence and size (208 lines) | 0.00 |
| 31 | `rg` six required section headings — structure check (six matches) | 0.00 |
| 32 | guarded `rg` token scan — prohibited-token absence | 0.00 |
| 33 | `git status --short --branch` — final read-only worktree check (`## main`) | 0.00 |

Verification after report creation checked the report structure and content, confirmed the prohibited token was absent, and confirmed that the repository worktree remained unchanged. Rows 30–33 record the commands, observed results, successful exit status, and timings.

Deviations from the brief: **NONE**. No repository file was edited; no toolchain, network, server, sub-worker, or Git state-changing operation was used.
