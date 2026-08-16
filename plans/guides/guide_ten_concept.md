# Draft: Guide 10 Concept — STATE Constructor and Exact Wide-Arithmetic Prototypes

> **Status:** Concept draft; not an execution guide
> **Phase:** Phase 3 — Elements target and foundational prototypes
> **Entry:** Recorded Guide-9 target-native primitive gate, plus closure of the blocking second-review findings below
> **Primary research owners:** [STATE constructor](../research/state-constructor.md), [wide arithmetic](../research/wide-arithmetic.md)
> **Affected packages:** `target-elements`, `tapscript`, `target-elements-conformance`
> **Does not complete:** Phase 3; public declassification remains a separate required prototype
> **Does not implement:** an attestation-contract operation, linked bundle, transaction ABI, calibrated bound, or release artifact

---

## Mission · `sec:guide10-concept:mission`

Guide 10 determines whether the reviewed Elements tapscript substrate can support two load-bearing backend mechanisms without weakening their target-independent semantics:

```text
metadata-dependent constructor continuity

exact wide floor arithmetic
    q = floor(a·b/d)
```

The guide must produce executable, target-native answers to two questions.

### Constructor question

Can a target program authenticate a metadata-dependent predecessor object, derive a successor metadata value, and require the successor to preserve the same authenticated static program relation?

Conceptually:

```text
predecessor:
    Constructor(static code root, metadata_before)

successor:
    Constructor(the same static code root, metadata_after)
```

The program must reject an independently chosen successor code root, wrong metadata, wrong internal key, wrong constructor schema, wrong tree composition, wrong output program, or spendable metadata escape.

### Arithmetic question

Can a target program prove exactly:

\[
q=\left\lfloor\frac{a·b}{d}\right\rfloor
\]

under the realization amount bounds:

```text
0 ≤ a,b,q < 2^51
0 < d < 2^51
```

without allowing:

- target signed overflow;
- an unchecked arithmetic-success flag;
- malformed or noncanonical limbs;
- an under-quotient;
- an over-quotient;
- a wrong remainder;
- a proof about values other than the enclosing operation’s authenticated operands.

Guide 10 is prototype-only. Its result is an accepted construction decision, a measured target rejection, or a sharply bounded residual question. It is not a production backend milestone by itself.

---

## Completion boundary · `rule:guide10-concept:completion-boundary`

Guide 10 succeeds when each research question reaches one of two typed outcomes:

```text
AcceptedPrototype
    exact construction identified
    target-native positive and negative evidence passed
    resource measurements recorded
    permanent handoff stated

TargetRejected
    no reviewed candidate satisfied the fixed semantic relation
    exact failing target boundary recorded
    no weaker semantic substitute silently adopted
```

A prototype is not accepted merely because:

- host-side code computes the desired result;
- a typed program serializes;
- the abstract stack validator accepts it;
- a mock executor agrees;
- one positive target transaction accepts;
- a target program fits in isolation;
- the report says “passed” without exact fixture and claim coverage;
- the construction works only with private data unavailable to its eventual caller.

The target-native report must bind the exact program, stack, transaction context, enforcement layer, target contract, deployment observation, and claim census.

---

# 1. Executive rulings · `sec:guide10-concept:rulings`

## 1.1 Repair the evidence boundary before recording new prototype evidence

The second static review found that the current native report and gate do not establish exact report completeness. Guide 10 must not build new prototype claims on that boundary unchanged.

Before any Guide-10 prototype report may satisfy a gate, the implementation must provide:

```text
raw native report
    ↓
complete owner validation
    ↓
validated native report wrapper
    ↓
prototype gate
```

The validator must require exact, duplicate-sensitive equality among:

```text
fixture case census
reported case census

evidence-plan row census
reported evidence row census

required claim census
passed claim census
```

The validated report must recompute rather than trust:

- case statuses;
- evidence dispositions;
- summary counts;
- report completeness.

Removing a failed row, relabeling it unresolved, clearing the evidence array, or editing the summary must fail validation.

## 1.2 Reports bind complete fixtures, not ordinal case names

A native case ordinal is local navigation inside one fixture census. It is not a semantic evidence identity.

Every prototype report row must carry, directly or through an admitted typed identity, the complete fixture subject:

- exact script bytes;
- exact initial stack;
- exact transaction context;
- consensus or relay enforcement layer;
- execution domain;
- leaf version and reviewed status;
- expected verdict;
- expected stack where statically fixed;
- expected resource observations;
- target and development binding.

Guide 10 should initially embed the complete typed fixture projection. It must not mint a fixture-set digest merely to reduce report size.

## 1.3 Broad evidence rows require exact subclaim coverage

One passing case is not sufficient to pass an evidence family.

Guide 10 introduces typed evidence claims below broad evidence requirement IDs. For example:

```rust
pub enum NativeEvidenceClaim {
    StackManipulationSuccess,
    StackManipulationUnderflow,

    ByteEqualitySuccess,
    ByteEqualityFailure,
    VerifySuccess,
    VerifyAbort,

    TapleafHashVector,
    TapbranchLeftOrder,
    TapbranchRightOrder,
    TaptweakEvenResult,
    TaptweakOddResult,

    ConstructorPredecessorBinding,
    ConstructorSuccessorBinding,
    ConstructorStaticRootContinuity,
    ConstructorMetadataMutationRejected,
    ConstructorStaticRootMutationRejected,
    ConstructorEscapePathRejected,

    WideFloorExactDivision,
    WideFloorNonzeroRemainder,
    WideFloorUnderQuotientRejected,
    WideFloorOverQuotientRejected,
    WideFloorZeroDivisorRejected,
    WideFloorMalformedLimbRejected,
    WideFloorUncheckedFlagRejected,
}
```

Equivalent factoring is acceptable.

For each evidence family \(e\), require:

\[
\operatorname{requiredClaims}(e)\subseteq\bigcup_{\substack{c\text{ passed}\\c\text{ bears on }e}}\operatorname{claims}(c).
\]

A claim without a passed case remains unresolved. It must not inherit success from another case in the same broad family.

## 1.4 Generic validation and exact reviewed binding stay distinct

A development binding validated against target definition \(A\) must not later combine with target definition \(B\) merely because both carry the same contract-version number.

Guide 10 requires one of:

```text
validated binding retains the complete validated target projection

or

a reviewed-development-binding wrapper is constructible only
against ReviewedElementsTapscriptDefinition
```

Native fixtures and reports must bind the exact target definition used to validate the deployment binding.

Version equality is insufficient.

## 1.5 The executed chain is observed, not caller-labelled

The native executor must report the actual development environment it executed:

- chain/environment class;
- observed genesis identity;
- observed network identity under one typed recipe;
- relevant activation state;
- reviewed leaf-version availability.

The harness compares those observations with the binding before accepting any case.

Synthetic run labels must not inhabit fields called `network_id` or `genesis_id`.

## 1.6 The executor process tree is supervised as one run

The executor is caller-selected code and is not sandboxed. Guide 10 nevertheless enforces its own timeout and cleanup contract.

On timeout, the harness must terminate the complete executor process group, including an adapter-spawned node, rather than only killing the immediate child. A timed-out run must not leave:

- an orphaned node;
- a live protocol pipe;
- a temporary node data directory;
- a disposable RPC cookie;
- a listening port.

Protocol lines also receive explicit byte limits. An executor must not force unbounded allocation by writing one unterminated line.

## 1.7 Signature abstraction is repaired before signature-dependent constructor integration

The current static signature model cannot represent its own documented empty-signature and unknown-key-type behavior exactly.

Before a Guide-10 result is used by an operator-authorized STATE operation, the operand and failure model must distinguish at least:

```text
empty signature
recognized nonempty valid signature
recognized nonempty invalid signature
empty public key
recognized public key
unknown nonempty public-key type
```

A type stating “exactly 64-byte signature” cannot simultaneously represent an empty signature failure path.

A type stating “exactly x-only public key” cannot represent the documented unknown-key-type compatibility path.

The standalone constructor prototype may remain signature-free. Phase-6 operation integration may not.

## 1.8 The target primitive substrate is expanded only by reviewed need

The current reviewed instruction subset is sufficient to describe individual Guide-9 primitives, but it does not yet obviously contain enough ordinary stack, equality, verification, control-flow, and byte operations to express the Guide-10 compound proofs.

Guide 10 begins with an explicit primitive-needs census. Candidate requirements include:

```text
stack:
    duplicate
    swap/reorder
    remove
    indexed copy where necessary

verification:
    equality
    equality-and-abort
    boolean verification
    conditional selection where unavoidable

bytes:
    exact concatenation or streamed chunk hashing
    exact width checks
    canonical byte comparison where TapBranch ordering needs it

existing reviewed substrate:
    streaming SHA-256
    fixed-width arithmetic
    signed comparison
    numeric conversions
    input/output program introspection
    tweak verification
```

No opcode is assumed available because it is familiar from another chain or script version.

For every admitted primitive:

- review its exact target byte;
- execution domain;
- success stack relation;
- failure relation;
- encoding;
- resource cost;
- target-native positive and negative cases;
- evidence requirements;
- cross-contract welds.

If the necessary primitive is absent or unsupported, the affected candidate is rejected. The guide does not replace it with an unreviewed raw opcode.

## 1.9 Prototype code is incapable of silently becoming release code

Guide-10 artifacts must carry an explicit prototype state.

Suitable forms include:

```rust
pub struct ConstructorPrototypeProgram { ... }

pub struct WideFloorPrototypeProgram { ... }

pub enum PrototypeStatus {
    Experimental,
    AcceptedResearchResult,
}
```

The production backend API must not accept these values as emitted operation programs or linked bundle members.

No generic `RelocatableTapscriptBundle`, transaction ABI, or release artifact is created in this guide.

## 1.10 Exact host references and target programs are independent implementations

The production target pattern and expected-result oracle must not share the implementation of the property under test.

For wide arithmetic:

```text
target program:
    fixed-width operations and limb relations

host oracle:
    u128 or arbitrary-precision exact arithmetic
```

For constructor hashing and tweaking:

```text
target program:
    target instruction sequence

host oracle:
    independently reviewed target-compatible hash/tree/tweak implementation
    or published vectors plus a separately implemented constructor
```

The target executor’s answer must not be used to generate its own expected value.

## 1.11 No speculative identity is minted

Guide 10 introduces no:

```text
ConstructorPrototypeHash
WideArithmeticPatternHash
PrototypeFixtureSetHash
PrototypeReportHash
TargetProgramHash
```

Typed values and exact bytes are retained directly.

A future linker or release consumer may activate identities under ADR-016. Guide 10 does not anticipate that decision with reserved digest fields.

---

# 2. Entry conditions · `sec:guide10-concept:entry`

Guide 10 begins only when:

- the Guide-9 gate record exists;
- the starting tree is clean;
- the exact starting revision is recorded;
- the second-review findings are accepted into the backlog;
- the native-report boundary can no longer pass an incomplete census;
- the report binds complete fixtures;
- the development binding is tied to the exact reviewed target;
- the executor reports the chain it actually runs;
- protocol message sizes are bounded;
- timeout cleanup covers the full process group;
- required additional target primitives have completed source review or are explicitly recorded as missing.

Before editing:

```sh
git status --porcelain=v1 --untracked-files=all
git log -1 --oneline
```

A dirty tree is not accepted as the Guide-10 starting state.

## 2.1 Blocking review findings

The following second-review findings block native prototype evidence:

| Finding family | Required disposition |
|---|---|
| native report census completeness | fixed with validated report wrapper |
| claim-level evidence completeness | fixed with exact typed claim census |
| fixture/report subject binding | fixed with complete fixture projection |
| exact target/deployment binding | fixed |
| observed network/genesis binding | fixed |
| executor provenance required by ADR-018 | fixed or explicitly excluded from the evidence claim |
| process-tree timeout cleanup | fixed |
| protocol record limits | fixed |
| protocol blank-line behavior | fixed and documented |

The signature-abstraction finding blocks signature-dependent use but need not prevent a deliberately signature-free standalone constructor experiment. It must close before the STATE prototype is described as ready for `announce-maturity`.

The deployment-profile schema limitation may remain outside Guide 10 because no deployment release is constructed. Public APIs must nevertheless avoid describing schema 2 as production-release-valid.

The relation-ID/body weld should close before Guide 10 adds any compiler-visible target requirement or relation variant. A prototype that does not change realization or compiler scope may proceed independently, but the finding remains a Phase-3 correctness task.

---

# 3. Required reading and authority · `sec:guide10-concept:authority`

Repository policy:

```text
AGENTS.md
adr/010-command-line-output-contract.md
adr/011-toolchain-and-dependency-policy.md
adr/014-meson-lint-census-and-stamps.md
adr/015-public-data-and-execution-trust.md
adr/016-semantic-identities-and-evidence-binding.md
adr/017-path-scope-and-host-filesystem-trust.md
adr/018-upstream-elements-workspace.md
```

Accepted implementation decisions:

```text
plans/decisions/001-typed-rust-source.md
plans/decisions/003-tapscript-first.md
plans/decisions/004-translation-validation.md
plans/decisions/005-value-representation.md
plans/decisions/006-transaction-abi.md
plans/decisions/008-exact-certified-mathematics.md
```

Normative realization owners:

```text
docs/attestation/realization.md
    §4.2 arithmetic gadgets
    §10 representation conformance
    §11 translation discipline
    T3 witnessed division
    T7 committed state
    T9 identity binding
    T12 cross-UTXO obligations
    T13 certificate relation to emitted predicate
```

Relevant imported labels include:

```text
(`[RZ-sec:arithmetic:gadgets]`)
(`[RZ-rule:translation:division]`)
(`[RZ-rule:translation:bind]`)
(`[RZ-rule:translation:struct]`)
(`[RZ-rule:translation:cross-utxo]`)
(`[RZ-rule:translation:certificate-leaf]`)
(`[RZ-inv:invariant:succession]`)
(`[RZ-pin:pins:arith]`)
(`[RZ-pin:pins:ident]`)
(`[RZ-pin:pins:weld]`)
```

Research owners:

```text
plans/research/state-constructor.md
plans/research/wide-arithmetic.md
plans/research/public-declassification.md
```

The first two are Guide-10 subjects. Public declassification remains a separate required Phase-3 result.

No package parses these documents as semantic input.

---

# 4. Scope · `sec:guide10-concept:scope`

## 4.1 In scope

Guide 10 implements or resolves:

- native-report validation hardening required by the prototypes;
- exact fixture and evidence-claim binding;
- exact reviewed target/development binding;
- observed chain identity in the native protocol;
- executor process-group cleanup and message-size limits;
- source review for the minimum additional target primitives;
- typed target contracts for admitted additional primitives;
- target-native vectors for every admitted primitive;
- a generic metadata-dependent constructor prototype;
- an exact host constructor oracle;
- constructor continuity, tree, tweak, and escape mutations;
- at least one exact wide-floor target pattern;
- an exact host wide-floor oracle;
- target-native wide-floor vectors and property-selected cases;
- exact stack contracts and resource projections for both prototypes;
- deterministic prototype reports;
- accepted or rejected research conclusions;
- Phase-3 and backlog handoff.

## 4.2 Out of scope

Guide 10 must not implement:

- `announce-maturity`;
- any other attestation-contract operation program;
- production STATE metadata ABI;
- final constructor symbols or relocations;
- a linked bundle;
- a taptree policy for production operation leaves;
- transaction request types;
- transaction signing;
- wide arithmetic inside redemption, settlement, or cycle;
- architecture-bound calibration;
- public declassification;
- direct confidential burn or redemption;
- production network activation;
- deployment release;
- release evidence identity;
- production wallet or signer support.

A synthetic STATE-like metadata structure may be used to exercise field changes. It must not be published as the final STATE ABI.

---

# 5. Package ownership · `sec:guide10-concept:packages`

## 5.1 `target-elements`

`target-elements` owns any newly reviewed target facts:

- additional opcode identities and bytes;
- operand, success, and failure contracts;
- stack and byte encodings;
- resource costs;
- capability rows;
- evidence requirements;
- cross-contract welds.

It remains dependency-free.

No constructor formula, protocol metadata schema, amount bound, or attestation-contract operation enters this package.

## 5.2 `tapscript`

`tapscript` owns typed compound target patterns:

- constructor prototype program;
- wide-floor prototype program;
- stack contracts;
- typed witnesses;
- target instruction sequences;
- deterministic serialization;
- static resource projections;
- exact pattern-local validation.

The existing first-party dependency boundary remains:

```text
compiler
target-elements
```

The guide should avoid adding a third-party dependency to `tapscript`.

Prototype APIs must be visibly prototype-only.

## 5.3 `target-elements-conformance`

`target-elements-conformance` owns:

- generic compound target fixtures;
- executor protocol extensions;
- exact fixture projections in reports;
- typed evidence claims;
- prototype report validation;
- target-native execution;
- independent expected vectors where package ownership permits;
- process supervision;
- report/stamp mechanics.

It may consume typed prototype programs from `tapscript`.

It remains forbidden from depending on:

```text
architecture
realization
model
compiler
linker
transaction
vectors
release
artifacts
labels
```

The fixtures remain generic target fixtures. They do not name STATE, redemption, settlement, cycle, or any other attestation-contract operation.

## 5.4 Reference dependency

Constructor reference calculation may require a reviewed generic cryptographic implementation not currently in the dependency graph.

Candidates must be reviewed before adoption. Possible roles include:

- tagged SHA-256 reference;
- x-only/compressed public-key handling;
- tweak calculation;
- target-compatible taproot tree construction.

No dependency is added merely because it is convenient. The review records:

- exact source and version;
- enabled features;
- licence;
- MSRV;
- unsafe/FFI boundary;
- deterministic behavior;
- transitive graph;
- advisory status;
- why existing code is insufficient.

The reference implementation belongs in the conformance/prototype side, never in the dependency-free target contract.

---

# 6. Preliminary target primitive closure · `sec:guide10-concept:primitive-closure`

## 6.1 Question

What is the smallest additional reviewed instruction set needed to express both prototypes with exact stack behavior?

The answer must be derived from explicit pattern schedules, not from a wish list.

## 6.2 Candidate primitive classes

### Stack movement

Potential needs include:

```text
duplicate top item
duplicate several items
swap top items
rotate a short frame
copy an indexed item
drop a consumed proof-local item
```

Every admitted operation must state exact failure behavior on insufficient stack depth.

A generic indexed operation is not preferred merely to reduce script size. A small fixed stack vocabulary may be easier to audit and schedule deterministically.

### Equality and verification

Potential needs include:

```text
byte equality
byte equality with abort
boolean verification
numeric exact equality
```

An arithmetic-success flag must be consumed by a verification relation. Leaving a false flag on stack for a later caller to notice is not an accepted proof pattern.

### Conditional selection

A conditional primitive may be needed when a target rule has two valid forms determined by witness data, such as:

- TapBranch child order;
- target output-key parity;
- constructor-totality retry form.

Conditionals are admitted only when:

- both branches have exact stack contracts;
- branch joins agree;
- the condition itself is authenticated;
- the selection does not let a caller choose a different constructor meaning.

### Byte operations

Potential needs include:

```text
concatenation
slice/split
length
canonical byte ordering
```

Streaming SHA-256 may remove the need for concatenation by hashing canonical chunks in sequence. It does not remove the need to establish:

- exact chunk boundaries;
- exact field widths;
- canonical TapBranch child ordering.

### Existing reviewed primitives

The prototype should reuse existing reviewed primitives where they are exact:

```text
streaming SHA-256
fixed-width arithmetic
signed fixed-width comparison
script-number/fixed-width conversion
input/output program introspection
elliptic-curve scalar verification
tweak verification
```

## 6.3 Primitive admission test

A primitive is admitted only when:

- its target byte is independently reviewed;
- its execution domain is reviewed;
- every success alternative is typed;
- every failure path is typed;
- its abstract stack transfer is exact;
- its resource cost is reviewed;
- native positive and negative vectors pass;
- its encoding and evidence welds pass;
- unknown forms fail closed.

A primitive absent from the reviewed target does not become a raw instruction in `tapscript`.

## 6.4 Decisive feasibility check

Before implementing either compound prototype, write a complete symbolic stack schedule for each leading candidate.

The schedule must show:

- initial stack;
- every instruction;
- stack after every success path;
- stack after every non-aborting failure path;
- every abort cause;
- peak main stack;
- peak alternate stack;
- largest item;
- values retained for the enclosing relation.

If the schedule needs an unavailable operation, the candidate is blocked at that exact step. Do not discover the missing primitive after a large implementation has landed.

---

# 7. STATE constructor prototype · `sec:guide10-concept:constructor`

## 7.1 Fixed semantic relation

The prototype models one metadata-dependent constructor:

```text
Constructor(M, C, P, S)
```

where:

- \(M\) is canonical public metadata;
- \(C\) is one authenticated static code-root value;
- \(P\) is one fixed public internal key;
- \(S\) is one constructor schema and target recipe.

The accepted transition relation is:

```text
consume Constructor(M_before, C, P, S)

derive M_after through one synthetic public transition

create Constructor(M_after, C, P, S)
```

The same \(C\), \(P\), and \(S\) must bind both sides.

The caller may supply witness data needed to reconstruct the constructor. The witness must not be free to select a different static relation.

## 7.2 Leading candidate

The leading research candidate remains the dynamic metadata leaf beside a static code subtree:

```text
static operation subtree root:
    C

dynamic metadata leaf:
    L(M)

tree root:
    R(M) = branch(C, L(M))

target output program:
    Q(M) = tweak(P, R(M))
```

The exact target formulas are review outputs, not assumptions. Conceptually, if the reviewed target follows tagged taproot construction:

\[
h_{\mathrm{leaf}}(M)=H_{\mathrm{TapLeaf}}(v\parallel\operatorname{compactSize}(|s(M)|)\parallel s(M))
\]

\[
h_{\mathrm{branch}}(C,h)=H_{\mathrm{TapBranch}}(\min(C,h)\parallel\max(C,h))
\]

\[
t(M)=H_{\mathrm{TapTweak}}(P\parallel h_{\mathrm{branch}}(C,h_{\mathrm{leaf}}(M)))
\]

\[
Q(M)=P+t(M)G
\]

Every domain tag, framing rule, child-ordering rule, scalar rule, parity rule, and output-program encoding must come from the reviewed target contract and native evidence.

## 7.3 Prototype metadata

Stage 1 uses synthetic fixed-width metadata:

```text
domain tag
schema number
object-kind test tag
counter
flags
```

with the transition:

```text
counter_after = counter_before + 1
all other fields unchanged
```

The metadata is deliberately STATE-like:

- more than one field;
- fields with independent mutation tests;
- explicit schema;
- canonical order;
- fixed width where possible.

It is not the final STATE ABI.

A later Guide-10 stage may run representative full STATE-shaped bytes as opaque canonical metadata, but no public type freezes them as the production transaction schema.

## 7.4 Predecessor authentication

The prototype program must establish:

1. the current input is the constructor instance being spent;
2. predecessor metadata is canonical;
3. the predecessor metadata leaf is exactly derived from those bytes;
4. the witnessed static root \(C\) participates in the predecessor constructor;
5. the fixed internal key \(P\) participates;
6. the predecessor target output program equals the reconstructed constructor;
7. the executing path is an admitted static operation path, not the metadata path.

Input-program introspection must read the exact consensus program.

An unauthenticated caller-supplied predecessor target key is insufficient.

## 7.5 Successor authentication

From authenticated predecessor metadata, the prototype derives the successor metadata and requires:

1. canonical successor encoding;
2. exact synthetic state delta;
3. the same static root \(C\);
4. the same internal key \(P\);
5. the same constructor schema \(S\);
6. the exact successor tree composition;
7. the exact successor target output program;
8. the exact output position or generic fixture role selected by the prototype.

No second static root is accepted as “the successor root”.

## 7.6 TapBranch ordering

Canonical child ordering is a decisive seam.

The prototype must establish the target’s exact TapBranch ordering. Acceptable strategies include:

- target byte-comparison primitives;
- a verified orientation witness whose correctness is itself checked;
- a target primitive that verifies the complete branch relation;
- another reviewed exact construction.

It is not sufficient to:

- hash children in caller order;
- accept either ordering without proving it corresponds to the target tree;
- derive one order off-chain and trust a bit without target validation.

If no reviewed mechanism can establish canonical child ordering, the dynamic-metadata-leaf candidate is rejected or revised.

## 7.7 Metadata leaf unspendability

The metadata leaf must not be an escape path.

The prototype must provide a target-native case showing:

```text
operation leaf:
    accepted under valid conditions

metadata leaf:
    cannot authorize a spend
```

The unspendable form itself must be reviewed and typed. A comment saying that a leaf is “data only” is not evidence.

## 7.8 Internal-key policy

The prototype uses a deterministic public point with no known private scalar.

It must not use:

- operator key;
- release key;
- test signing key;
- generated-and-discarded private key;
- mutable deployment key.

The report states the exact derivation or published constant.

The nonexistence of a private scalar is a cryptographic assumption, not something the prototype proves. The accepted decision must state that residual honestly.

## 7.9 Tweak totality

The target constructor must define what happens when the tweak scalar or point construction is invalid.

Candidates include:

```text
reject the constructor instance

canonical metadata representation nonce:
    try nonce 0,1,2,... under a bounded deterministic rule

canonical public internal-key retry

accept a named negligible constructibility residual
```

Silence is not a policy.

If a retry is selected, it must be:

- canonical;
- publicly computable;
- bounded or total under a reviewed argument;
- part of the constructor schema;
- reflected in the fixture and resource model;
- incapable of changing semantic metadata.

## 7.10 Constructor candidates

The prototype should compare at least:

| Candidate | Description | Acceptance concern |
|---|---|---|
| Dynamic metadata leaf | Static root plus unspendable metadata leaf | Branch ordering, tweak totality, metadata path |
| Separate metadata output | Fixed program plus separately welded metadata object | Adds paired state and may require normative change |
| Witnessed root continuity | Witness one static root and verify both constructor instances | Must prove root is the one encoded by each target program |
| Metadata in every operation leaf | Every operation leaf commits metadata | Recreates recursive/static-code problem and likely larger trees |

The separate-metadata-output candidate is a semantic architecture change if adopted for production. It may be measured as a comparison but cannot be accepted by Guide 10 without upstream realization and architecture review.

## 7.11 Constructor threat matrix

Required focused mutations include:

| Mutation | Required result |
|---|---|
| predecessor metadata field changed | reject |
| successor metadata field changed | reject |
| unchanged counter where increment required | reject |
| increment by two | reject |
| malformed schema | reject |
| alternate canonical encoding | reject |
| trailing metadata bytes | reject |
| wrong metadata domain | reject |
| wrong metadata leaf version | reject |
| wrong static root | reject |
| predecessor and successor use different static roots | reject |
| wrong internal key | reject |
| wrong output-key parity | reject |
| wrong TapLeaf hash | reject |
| wrong TapBranch order | reject |
| wrong TapTweak input | reject |
| successor output in the wrong role | reject |
| operation leaf removed from static subtree | reject |
| extra escape leaf introduced | reject |
| metadata leaf selected for spend | reject |
| constructor from another prototype schema | reject |
| stale constructor recipe | reject |
| target program correct but semantic successor wrong | reject |

The last row proves that constructor continuity and semantic state assignment remain separate obligations.

---

# 8. Exact wide-floor prototype · `sec:guide10-concept:wide-floor`

## 8.1 Fixed semantic relation

The prototype proves:

\[
a·b=q·d+r
\]

with:

\[
0\le r<d,\qquad 0\le a,b,q<2^{51},\qquad 0<d<2^{51}.
\]

These conditions imply:

\[
q=\left\lfloor\frac{a·b}{d}\right\rfloor.
\]

The target proof must establish the complete relation. A host-generated \(q\) and \(r\) are witnesses, not trusted answers.

## 8.2 Exact host oracle

The reference oracle computes with exact arithmetic:

```rust
product = a·b
q = product / d
r = product % d
```

Because \(a,b<2^{51}\), the product is below \(2^{102}\) and fits in `u128`.

The host oracle should still have property coverage against arbitrary-precision arithmetic where practical, so a later widening of the domain cannot silently invalidate the reference implementation.

The host oracle returns:

- \(q\);
- \(r\);
- canonical operand limbs;
- canonical product limbs;
- every expected carry;
- exact range maxima used by the target proof.

## 8.3 Candidate A — derived limbs

The leading candidate uses base:

\[
B=2^{26}.
\]

Each semantic amount decomposes as:

\[
x=x_0+x_1B
\]

with:

```text
0 ≤ x0 < 2^26
0 ≤ x1 < 2^25
```

For:

\[
a=a_0+a_1B,\qquad b=b_0+b_1B,
\]

derive:

\[
c_0=a_0b_0
\]

\[
c_1=a_0b_1+a_1b_0
\]

\[
c_2=a_1b_1.
\]

Normalize through checked carries into four base-\(B\) limbs:

\[
P(a,b)=[p_0,p_1,p_2,p_3].
\]

Independently derive:

\[
P(q,d)
\]

and add the two-limb representation of \(r\), with checked carry propagation.

Finally require exact limb equality:

\[
P(a,b)=P(q,d)+r.
\]

The target derives operand limbs through exact target division by \(B\), rather than trusting caller-supplied limbs, if the measured program is feasible.

## 8.4 Candidate B — witnessed limbs and carries

The caller supplies:

```text
q
r
operand limbs
product limbs
carry witnesses
```

The target:

- checks every limb bound;
- recomposes every operand;
- checks every partial-product equation;
- checks every carry equation;
- checks exact product equality;
- checks remainder bounds.

This candidate increases witness size but may reduce script and stack complexity.

It is accepted only if every witness component is fully constrained. A witness item that may vary without changing target acceptance is either irrelevant and removed or is part of a deliberately typed equivalence class.

## 8.5 Candidate C — sandwich proof

Compare against:

\[
q·d\le a·b<(q+1)·d.
\]

This removes \(r\) from the witness but requires:

- three wide products or equivalent relations;
- two wide comparisons;
- careful handling of \(q+1\);
- exact high-limb comparison.

It should be prototyped only far enough to determine whether it is materially simpler than quotient/remainder. No preference is granted merely because the formula is shorter.

## 8.6 Range proof table

Before emitting a target program, derive exact symbolic maxima for every intermediate of Candidate A and Candidate B.

For \(B=2^{26}\):

```text
a0,b0,q0,d0 < 2^26
a1,b1,q1,d1 < 2^25

a0·b0              < 2^52
a0·b1              < 2^51
a1·b0              < 2^51
a0·b1 + a1·b0      < 2^52
a1·b1              < 2^50
```

The review must continue through:

- first carry;
- second normalized coefficient;
- second carry;
- top product limb;
- \(q·d\) coefficients;
- remainder decomposition;
- remainder addition carries;
- comparison temporaries;
- constants loaded into signed target arithmetic.

Every target operation must remain below \(2^{63}\) on every valid path.

“Each input is below \(2^{51}\)” is not itself an overflow proof.

## 8.7 Arithmetic success flags

Every fixed-width arithmetic operation that returns a success flag must have that flag verified immediately or under a typed schedule proving it cannot be lost, overwritten, or confused with a result.

Required form:

```text
operation
    ↓
result + success flag
    ↓
verify success flag
    ↓
retain only authenticated result
```

Forbidden forms:

- leave the flag for an unspecified later caller;
- compare the result while ignoring the flag;
- let retained operands from failure satisfy the succeeding stack contract;
- treat a non-aborting false as an abort;
- infer success because the final stack is nonempty.

The abstract stack validator must model the failure path and prove the enclosing fragment cannot continue from it as if success occurred.

## 8.8 Pattern output

The wide-floor pattern must have an exact typed postcondition.

Preferred form:

```text
input:
    authenticated a,b,d
    public q,r witness

success:
    q retained in one canonical 8-byte form
    no unchecked flag
    no residual proof-local stack item

failure:
    abort
```

If the pattern leaves only a Boolean and requires the caller to reintroduce \(q\), it risks proving one quotient while using another. The authenticated \(q\) should remain the value consumed by the eventual operation relation.

## 8.9 Wide-floor threat matrix

Required fixed and generated cases include:

| Mutation | Required result |
|---|---|
| exact quotient and remainder | accept |
| exact division, \(r=0\) | accept |
| \(r=1\) | accept |
| \(r=d-1\) | accept |
| \(q-1\) | reject |
| \(q+1\) | reject |
| correct equality with \(r=d\) | reject |
| zero divisor | reject |
| negative target operand | reject |
| operand equal to \(2^{51}\) | reject |
| quotient equal to \(2^{51}\) | reject |
| malformed 7-byte operand | reject |
| malformed 9-byte operand | reject |
| byte-reversed operand | reject |
| low limb one above bound | reject |
| high limb one above bound | reject |
| wrong first carry | reject |
| wrong second carry | reject |
| wrong top limb | reject |
| omitted high limb | reject |
| duplicated limb | reject |
| witness items reordered | reject |
| one arithmetic flag unchecked | validation or target rejection |
| proof computed over substituted \(a\) | enclosing binding rejection |
| proof computed over substituted \(d\) | enclosing binding rejection |

Boundary values include:

```text
0
1
B-1
B
B+1
2^25-1
2^25
2^26-1
2^51-2
2^51-1
```

## 8.10 Property strategy

Host property tests generate:

```text
a,b in [0,2^51)
d in [1,2^51)
```

and derive exact \(q,r\).

For each generated valid case, test at least:

```text
(q,r)
(q-1, adjusted or unchanged r)
(q+1, adjusted or unchanged r)
wrong r
one malformed limb or carry
```

Target-native execution uses a deterministic selected subset:

- all fixed boundaries;
- all carry-shape classes;
- all quotient classes;
- deterministic seeded generated cases;
- minimized regressions from any discovered mismatch.

A huge random native suite is not a substitute for a complete semantic partition of the input space.

---

# 9. Generic compound-prototype fixtures · `sec:guide10-concept:fixtures`

## 9.1 New fixture class

Primitive fixtures and compound-prototype fixtures should remain distinct typed values.

Conceptually:

```rust
pub struct CompoundPrototypeFixture {
    pub case: PrototypeCaseId,
    pub claim: PrototypeClaim,
    pub program: TapscriptProgram,
    pub initial_stack: Vec<StackItem>,
    pub context: PrimitiveExecutionContext,
    pub expected: ExpectedPrototypeOutcome,
    pub resources: ExpectedResourceObservation,
}
```

The fixture is still target-generic. Its claims may say:

```text
metadata constructor continuity
wide floor relation
```

but must not name an attestation-contract operation, STATE root, receipt, pool, or semantic relation ID.

## 9.2 Taptree materialization

The current executor constructs a single-leaf tree. The constructor prototype requires a target-generic tree capable of representing:

```text
executing operation leaf
metadata sibling leaf
optional static subtree root or leaves
```

Extend the fixture language with a typed tree description, for example:

```rust
pub enum FixtureTapTree {
    Leaf {
        version: u8,
        script: Vec<u8>,
    },
    Branch {
        left: Box<FixtureTapTree>,
        right: Box<FixtureTapTree>,
    },
}
```

or an equivalent canonical leaf/path representation.

The executor must materialize exactly that tree and return or verify:

- output program;
- control path;
- executing leaf;
- leaf version;
- internal key;
- branch composition.

A fixture-provided control path is not trusted without reconstructing and checking the target program.

## 9.3 Stated and supplied fields

The fixture language must continue to distinguish:

```text
stated field:
    exact requirement the executor must materialize

absent field:
    value supplied by the executor under a documented rule
```

The executor must reject, not approximate, a stated field it cannot materialize.

For constructor fixtures, the following should normally be stated:

- internal key;
- tree structure;
- leaf versions;
- exact leaf scripts;
- predecessor program;
- successor program;
- transaction output role;
- metadata bytes.

Funding outpoints may remain executor-supplied.

## 9.4 Report subject

Each report case embeds the complete fixture projection and records:

- exact observed environment;
- exact executor provenance;
- exact target verdict;
- observed failure class where available;
- observed transaction weight;
- unavailable interpreter observations as absent, never zero.

---

# 10. Prototype evidence and reports · `sec:guide10-concept:evidence`

## 10.1 Report classes

Use separate report classes:

```text
target primitive report
constructor prototype report
wide-floor prototype report
```

A primitive report does not satisfy a constructor claim.

A constructor report does not satisfy a wide-arithmetic claim.

No report is a deployment release report.

## 10.2 Validated report wrapper

A compound prototype report becomes gate-eligible only after validation against:

- reviewed target;
- exact reviewed development binding;
- observed chain identity;
- exact fixture set;
- exact claim plan;
- exact case census;
- exact executor transcript.

Conceptually:

```rust
pub struct ValidatedPrototypeReport {
    report: PrototypeReport,
}
```

Only this wrapper enters the Guide-10 gate.

## 10.3 Completeness states

```rust
pub enum PrototypeReportCompleteness {
    CompleteForConstructorPrototype,
    CompleteForWideFloorPrototype,
    PartialUnresolvedClaims,
    Failed,
}
```

A report with unresolved required claims cannot use a complete status.

A report with an infrastructure error is failed, not partial.

## 10.4 Provenance

Record separately:

- first-party harness revision;
- adapter implementation and version;
- functional-test framework revision;
- node implementation and binary-reported revision;
- intended executed integration tip;
- upstream base;
- included local topic branches;
- development network and genesis;
- activation observation.

Checkout `HEAD` must not be substituted for a binary-reported revision.

## 10.5 Determinism

Given equal:

- reviewed target;
- exact binding;
- fixture set;
- executor observations;
- claim plan;
- work limits;

report bytes are equal.

Exclude:

- wall clock;
- elapsed time;
- host;
- username;
- process ID;
- temporary paths;
- raw executor path;
- environment values;
- protocol scheduling order.

---

# 11. Independent oracles · `sec:guide10-concept:oracles`

## 11.1 Primitive-byte oracle

Every newly admitted primitive has an explicit expected-byte table independent of the production registry.

## 11.2 Stack-relation oracle

A separately written oracle states:

- operands;
- success alternatives;
- consumed and retained operands;
- non-aborting failure states;
- abort causes;
- resource cost.

It must include the compound sequences’ needed ordinary stack and verification primitives.

## 11.3 Constructor oracle

The constructor oracle computes:

- canonical metadata bytes;
- metadata leaf script;
- leaf hash;
- static/dynamic branch root;
- tweak;
- output key/program;
- control path.

It must not call the production tapscript constructor builder.

Where possible, compare three values:

```text
first-party independent host reference
reviewed upstream/library constructor
target-native spend result
```

A mismatch is triaged. The target observation is not copied into the expected value.

## 11.4 Wide arithmetic oracle

The exact host oracle uses `u128` or arbitrary-precision arithmetic and directly checks:

\[
a·b=q·d+r
\]

and:

\[
0\le r<d.
\]

A separate small reference should verify the limb normalization rather than call the production limb helper.

## 11.5 Abstract execution oracle

For bounded short programs and selected compound fragments, enumerate every compatible stack transition independently and compare complete:

- success state set;
- non-aborting failure state set;
- abort-cause set.

---

# 12. Security boundary · `sec:guide10-concept:security`

## 12.1 Public data only

Guide-10 inputs are public test data:

- metadata bytes;
- static tree roots;
- public internal keys;
- target programs;
- quotient and remainder witnesses;
- public arithmetic values;
- disposable development transactions;
- explicit executor capability.

No interface accepts:

- production private key;
- wallet seed;
- signing nonce;
- production blinding factor;
- private opening;
- RPC credential;
- bearer token;
- cookie path;
- production endpoint.

## 12.2 Test cryptographic values

Published curve vectors, public NUMS points, and disposable test-network values are public fixture material.

They must be clearly test-only and must not be reused for production authority.

## 12.3 Executor authority

Selecting the executor grants execution authority.

The harness does not authenticate or sandbox it.

The environment remains:

- secretless;
- disposable;
- isolated externally;
- free of production or repository-write authority where untrusted source runs.

## 12.4 Diagnostics

Do not emit:

- raw child argv;
- raw child stderr;
- executor path;
- environment values;
- temporary directory;
- cookie path.

Typed safe provenance and case IDs are permitted.

## 12.5 Crash artifacts

No repository-controlled lane publishes core dumps, heap captures, or node data directories.

---

# 13. Resource evidence · `sec:guide10-concept:resources`

## 13.1 Constructor measurements

Record:

- metadata bytes;
- operation leaf bytes;
- metadata leaf bytes;
- static subtree representation;
- control-path bytes;
- predecessor-verification script bytes;
- successor-verification script bytes;
- witness bytes;
- transaction weight;
- peak abstract stack;
- peak abstract alternate stack;
- largest element;
- number of hash operations;
- number of tweak/curve operations;
- validation budget.

Measure at least:

- minimum metadata;
- representative metadata;
- maximum prototype metadata;
- both branch-order cases;
- both output-key parity cases;
- wrong-root rejection;
- metadata-leaf escape rejection.

## 13.2 Wide-floor measurements

Record for each candidate:

- script bytes;
- witness items and bytes;
- arithmetic operation count;
- comparisons;
- divisions;
- verification operations;
- peak abstract stack;
- peak alternate stack;
- largest element;
- transaction weight in a generic fixture;
- target/policy verdict.

Measure separately:

- exact division;
- maximal carry propagation;
- maximal operand values;
- quotient zero;
- largest accepted quotient;
- rejected under-quotient;
- rejected over-quotient;
- malformed witness.

## 13.3 No architecture calibration

The measurements answer:

```text
Is the prototype plausible?
Which candidate is smaller or clearer?
Which target limit is first?
```

They do not select:

- `SETTLEMENT_BATCH_MAX`;
- any burn/transfer bound;
- a production tree depth;
- final operation resources.

No architecture default changes as a result of an isolated prototype measurement.

---

# 14. Acceptance criteria · `sec:guide10-concept:acceptance`

## 14.1 Constructor acceptance

Accept one constructor candidate only when:

- predecessor program is reconstructed from authenticated metadata, static root, internal key, and schema;
- successor program is reconstructed from exact successor metadata;
- one authenticated static root binds both sides;
- canonical TapLeaf and TapBranch rules are target-enforced;
- internal-key and parity rules are exact;
- metadata leaf is unspendable;
- key path supplies no known authority;
- tweak totality policy is explicit;
- every wrong-root/key/metadata/schema/path mutation rejects;
- native and abstract stack results agree;
- target resource measurements fit the prototype limits;
- output bytes are deterministic;
- no manual post-serialization patch is used;
- prototype status remains explicit.

## 14.2 Wide-floor acceptance

Accept one arithmetic candidate only when:

- floor equivalence is mathematically exact;
- \(d>0\) is enforced;
- \(q\) and \(r\) are fully constrained;
- every semantic operand is range-checked;
- every limb and carry is constrained canonically;
- every intermediate stays inside signed target range;
- every arithmetic success flag is checked;
- \(q-1\) and \(q+1\) reject for every eligible positive fixture;
- malformed widths, signs, limbs, carries, and witness order reject;
- target-native and host-reference results agree;
- the exact authenticated quotient remains available to the enclosing relation;
- target resource measurements are recorded;
- settlement batch feasibility is not claimed from the standalone pattern.

## 14.3 Evidence acceptance

No prototype is accepted unless:

- exact fixture census validates;
- exact claim census validates;
- every required claim has a passed case;
- no required case failed;
- no required case hit infrastructure trouble;
- report subject binding is complete;
- observed chain matches the binding;
- executor provenance meets ADR-018 or the report explicitly does not claim that provenance class;
- report bytes reproduce;
- the gate accepts only a validated report wrapper.

---

# 15. Rejection criteria · `sec:guide10-concept:rejection`

Reject a constructor candidate if it:

- trusts metadata without binding it to the consumed program;
- accepts independent predecessor and successor static roots;
- cannot enforce canonical branch ordering;
- leaves a spendable metadata or key path;
- relies on a secret for a future permissionless operation;
- has an undefined tweak-failure case;
- works only through manually patched bytes;
- cannot be materialized by the generic executor;
- passes only under a mock;
- exceeds target limits in the standalone prototype;
- changes the semantic object model without upstream review.

Reject a wide-floor candidate if it:

- accepts \(q-1\) or \(q+1\);
- accepts \(r\ge d\);
- accepts \(d=0\);
- leaves one high limb or carry unchecked;
- can overflow any target intermediate on a valid semantic input;
- leaves an arithmetic false flag unverified;
- proves a quotient distinct from the one later used;
- requires an unavailable private witness;
- passes only under host arithmetic or the abstract validator;
- differs between tested and proposed production bytes.

Reject the Guide-10 gate if:

- any required native claim is absent;
- a report can pass after deleting a failed row;
- fixture bytes are not present or bound;
- network/genesis are caller labels rather than observations;
- timeout can leak the executor’s node;
- an oversized protocol record can force unbounded allocation;
- a skipped native lane is described as passed.

---

# 16. Suggested implementation waves · `sec:guide10-concept:waves`

## Wave 0 — Accept and close the blocking review register

Deliver:

- validated native-report wrapper;
- exact fixture projection in reports;
- typed evidence claims;
- exact target/development binding;
- observed native environment;
- complete executor provenance;
- process-group timeout cleanup;
- bounded protocol messages;
- strict protocol framing;
- signature operand/failure model repair;
- relation-ID/body weld if compiler-visible work will follow.

Suggested commit:

```text
target-conformance: make native evidence exact and self-validating
```

## Wave 1 — Review the missing compound-proof primitives

Deliver:

- complete primitive-needs census;
- source review;
- typed target contracts;
- cross-contract welds;
- opcode-byte oracle;
- abstract stack oracle;
- native positive and negative cases.

Suggested commit:

```text
target-elements: review the compound-proof instruction substrate
```

Stop here if the minimum substrate is unavailable.

## Wave 2 — Extend generic fixture tree materialization

Deliver:

- typed generic taptree fixture;
- exact leaf/control materialization;
- stated versus supplied field rules;
- output-program observations;
- constructor-vector fixture support;
- no attestation-contract semantics.

Suggested commit:

```text
target-conformance: materialize generic constructor trees
```

## Wave 3 — Constructor host oracle

Deliver:

- canonical prototype metadata codec;
- independent tagged-hash/tree/tweak oracle;
- published-vector cross-checks;
- child-order and parity cases;
- totality-policy candidates.

Suggested commit:

```text
target-conformance: add the constructor reference oracle
```

## Wave 4 — Readable constructor target prototype

Deliver:

- predecessor binding;
- successor reconstruction;
- same-static-root continuity;
- metadata-leaf unspendability;
- exact stack contract;
- all focused mutations;
- resource projection.

Suggested commit:

```text
tapscript: prototype metadata-dependent constructor continuity
```

## Wave 5 — Exact wide-floor host oracle and bound proof

Deliver:

- exact \(q,r\) oracle;
- base-\(2^{26}\) decomposition;
- symbolic intermediate maxima;
- independent normalization oracle;
- fixed and generated vectors.

Suggested commit:

```text
tapscript: establish the exact wide-floor reference
```

## Wave 6 — Wide-floor target candidates

Deliver:

- Candidate A derived-limb pattern;
- Candidate B only if Candidate A needs comparison;
- Candidate C only if it plausibly simplifies the relation;
- exact success-flag handling;
- target-native vectors;
- resource measurements;
- deterministic selected candidate or explicit rejection.

Suggested commit:

```text
tapscript: prototype exact wide floor verification
```

## Wave 7 — Research decisions and gate

Deliver:

- updated STATE-constructor research result;
- updated wide-arithmetic research result;
- package and phase documentation;
- compact gate record;
- identity and dependency impact;
- complete repository verification;
- clean final tree.

Suggested commit:

```text
plans: record the Guide-10 prototype decisions
```

Commit each coherent green wave promptly.

---

# 17. Focused verification · `sec:guide10-concept:verification`

## 17.1 Target contract

```sh
cargo test --locked -p tripod-target-elements
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-target-elements --no-deps
```

Required focused areas:

```text
new primitive bytes
success alternatives
failure alternatives
stack-resource welds
signature operand cases
capability prerequisite closure
reviewed-target trust state
```

## 17.2 Tapscript prototypes

```sh
cargo test --locked -p tripod-tapscript
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-tapscript --no-deps
```

Required focused areas:

```text
constructor stack schedules
constructor program determinism
wrong-root/key/metadata programs
wide-floor host oracle
limb normalization
target arithmetic flags
q-1/q/q+1
abstract execution oracle
resource projection
```

## 17.3 Native conformance

```sh
cargo test --locked -p tripod-target-elements-conformance
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-target-elements-conformance --no-deps
```

Required focused areas:

```text
validated report wrapper
missing/duplicate/unexpected rows
claim-level coverage
fixture subject binding
observed chain identity
ADR-018 provenance
bounded protocol records
process-group timeout
tree materialization
constructor vectors
wide-floor vectors
report determinism
```

## 17.4 Native execution

Use the reviewed explicit executor through the non-default native lane.

The final command records:

- executor adapter;
- node binary version;
- binary-reported revision;
- intended integration tip;
- upstream base;
- included local topics;
- development chain/genesis;
- exact report and stamp paths.

No credential appears in the command.

## 17.5 Documentation and census

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Every new tracked source joins its nearest `meson.build` census in the same commit.

---

# 18. Full batch gate · `gate:guide10-concept:batch`

After the coherent batch:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

Run the real target-native constructor and wide-floor matrices separately with the reviewed executor.

Run:

```sh
cargo audit
```

when installed. If unavailable, record it as skipped, never passed.

Run document byte reproducibility only when required by changed document inputs or the repository’s batch policy. If deferred, record it as deferred.

Final check:

```sh
git status --porcelain=v1 --untracked-files=all
```

The output must be empty.

---

# 19. Identity and dependency impact · `sec:guide10-concept:impact`

Expected identity impact:

```text
Attestation version:
    unchanged

realization version:
    unchanged

architecture schema:
    unchanged

architecture semantic hash:
    unchanged

architecture behavioural hash:
    unchanged

generated architecture publications:
    unchanged

realization identity:
    none minted

compiler identity:
    none minted

target-definition digest:
    none minted

prototype-program digest:
    none minted

prototype-report digest:
    none minted

deployment-profile identity:
    remains dormant
```

The target contract version may require a deliberate bump if newly admitted primitive semantics change the contract rather than complete a previously admitted incomplete field. The decision must be explicit.

Expected dependency impact:

```text
target-elements:
    none

tapscript:
    no new dependency expected

target-elements-conformance:
    possible reviewed generic hash/curve reference dependency,
    only if the independent constructor oracle has a concrete need
```

Any such dependency is reviewed under ADR-011 before adoption.

---

# 20. Guide-10 exit checklist · `gate:guide10-concept:exit`

Guide 10 exits only when all applicable items hold.

## Evidence foundation

- [ ] native report has a validated wrapper;
- [ ] evidence-row census is exact and duplicate-sensitive;
- [ ] fixture-case census is exact and duplicate-sensitive;
- [ ] each report row binds the complete fixture projection;
- [ ] broad evidence requirements have typed subclaim censuses;
- [ ] deleting or relabeling a failed row cannot produce a pass;
- [ ] report summary is recomputed;
- [ ] observed chain identity equals the validated binding;
- [ ] exact target definition is bound to the development binding;
- [ ] executor provenance records executed tip, upstream base, and local topics where claimed;
- [ ] timeout terminates the complete executor process tree;
- [ ] protocol record sizes are bounded;
- [ ] protocol blank-line behavior is strict and documented.

## Compound instruction substrate

- [ ] every required primitive has source review;
- [ ] every admitted opcode byte is independently checked;
- [ ] every success form is typed;
- [ ] every failure form is typed;
- [ ] ordinary stack operations have exact contracts;
- [ ] equality and verification behavior is exact;
- [ ] no raw unreviewed opcode exists;
- [ ] signature empty/invalid/unknown-key paths are representable before signature use;
- [ ] abstract and native primitive results agree.

## Constructor prototype

- [ ] canonical prototype metadata exists;
- [ ] predecessor metadata binds to predecessor program;
- [ ] successor metadata is derived exactly;
- [ ] one authenticated static root binds both sides;
- [ ] internal-key policy is explicit;
- [ ] TapLeaf framing is exact;
- [ ] TapBranch ordering is target-enforced;
- [ ] TapTweak construction is exact;
- [ ] parity handling is exact;
- [ ] metadata leaf is unspendable;
- [ ] key path carries no known authority;
- [ ] tweak totality policy is explicit;
- [ ] wrong metadata, root, key, schema, order, and path mutations reject;
- [ ] complete constructor fixtures run through the real target;
- [ ] constructor resource measurements are recorded;
- [ ] no production STATE ABI is frozen.

## Wide-floor prototype

- [ ] exact floor equivalence is documented;
- [ ] exact host oracle exists;
- [ ] intermediate range proof is complete;
- [ ] every target arithmetic flag is checked;
- [ ] operand and witness domains are exact;
- [ ] limb and carry constraints are complete;
- [ ] \(d=0\) rejects;
- [ ] \(q-1\) rejects;
- [ ] \(q+1\) rejects;
- [ ] \(r=d\) rejects;
- [ ] malformed widths, signs, limbs, carries, and witness order reject;
- [ ] abstract and native stack outcomes agree;
- [ ] fixed and deterministic generated vectors pass;
- [ ] target resource measurements are recorded;
- [ ] no operation-level feasibility is overclaimed.

## Boundary and documentation

- [ ] prototype types cannot enter release output;
- [ ] no linked bundle or ABI exists;
- [ ] no speculative identity was minted;
- [ ] no target-specific type flowed back into realization or compiler core;
- [ ] STATE-constructor research records accepted result or target rejection;
- [ ] wide-arithmetic research records accepted result or target rejection;
- [ ] public declassification remains visibly open;
- [ ] package READMEs are current;
- [ ] Phase-3 card is current;
- [ ] backlog records the exact gate;
- [ ] every new source is in the Meson census;
- [ ] complete Rust and Meson gates pass;
- [ ] real native matrices pass;
- [ ] skipped or deferred lanes are reported honestly;
- [ ] final repository tree is clean.

---

# 21. Completion report template · `sec:guide10-concept:report-template`

```text
Guide 10 result
===============

Starting state:
    source revision:
    Guide-9 gate:
    second-review register:
    clean tree:

Evidence preflight:
    validated report wrapper:
    fixture projection binding:
    exact case census:
    exact evidence-row census:
    claim-level coverage:
    summary recomputation:
    target/binding exactness:
    observed network/genesis:
    executor provenance:
    process-group cleanup:
    protocol size limit:
    protocol framing:
    signature abstraction:
    relation-ID/body weld:

Primitive closure:
    primitives required:
    primitives already reviewed:
    primitives newly reviewed:
    primitives unavailable:
    target-contract version impact:
    native primitive cases:

Constructor prototype:
    selected candidate:
    metadata schema:
    static code-root representation:
    internal key:
    leaf hash:
    branch hash:
    branch ordering:
    tweak relation:
    predecessor authentication:
    successor authentication:
    metadata-leaf unspendability:
    key-path residual:
    totality policy:
    positive cases:
    negative cases:
    abstract/native agreement:
    resources:
    decision:
        accepted / target rejected / unresolved

Wide-floor prototype:
    selected candidate:
    base:
    witness:
    exact oracle:
    intermediate bounds:
    arithmetic flag enforcement:
    q-1:
    q:
    q+1:
    zero divisor:
    malformed limb/carry:
    property cases:
    abstract/native agreement:
    resources:
    decision:
        accepted / target rejected / unresolved

Native executor:
    adapter:
    framework:
    node:
    binary-reported revision:
    intended executed tip:
    upstream base:
    local topics:
    chain:
    observed genesis:
    activation:
    exact command:

Reports:
    constructor cases:
    constructor passed:
    constructor failed:
    constructor infrastructure errors:
    constructor unresolved claims:
    wide-floor cases:
    wide-floor passed:
    wide-floor failed:
    wide-floor infrastructure errors:
    wide-floor unresolved claims:
    report bytes deterministic:

Identity impact:
    Attestation:
    realization:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    target contract version:
    prototype identities:
        none
    report identities:
        none
    deployment identity:
        none

Dependency impact:
    target-elements:
    tapscript:
    target-elements-conformance:
    Cargo.lock:
    licence:
    MSRV:
    unsafe/FFI:
    advisories:

Verification:
    cargo fmt --all:
    cargo clippy --workspace --all-targets --locked -- -D warnings:
    cargo test --workspace --locked:
    target-elements:
    tapscript:
    target-elements-conformance:
    Rustdoc:
    scripts/check-plans.sh:
    meson compile -C build lint:
    scripts/ci.sh:
    meson compile -C build:
    meson test -C build --print-errorlogs:
    real constructor matrix:
    real wide-floor matrix:
    cargo audit:
    document reproducibility:
    git diff --check:
    final git status:

Planning handoff:
    STATE constructor research:
    wide arithmetic research:
    Phase 3:
    next guide:

Residuals:
```

---

# 22. What follows Guide 10 · `sec:guide10-concept:next`

Guide 10 does not complete Phase 3.

If the constructor and arithmetic prototypes are accepted, the remaining foundational prototype is:

```text
Guide 11 — Public Declassification and Confidential-to-Public Lifecycle
```

That guide should decide:

- explicit-only boundary policy;
- owner-authorized normalization;
- direct authenticated opening;
- residual blinding closure;
- public committed ASH;
- fresh-process permissionless maintenance;
- sponsor-region conservation evidence;
- representation safety versus minimality.

Only after all three Phase-3 research owners have accepted results or explicit target rejections may production operation guides proceed.

The likely operation sequence remains:

```text
compact ASH
live transfer
STATE and maturity
burn and clear
redemption
requests and admission
settlement
cycle
release
```

Guide 10 supplies mechanisms to later phases. It does not move those phases forward by itself.

---

## One-line concept · `rem:guide10-concept:one-line`

> Guide 10 must prove, against an exact and self-validating native-evidence boundary, that Elements tapscript can preserve one metadata-dependent constructor across a state transition and can verify \(q=\lfloor a·b/d\rfloor\) exactly under the \(2^{51}\) amount domain—without emitting an attestation-contract operation, freezing an ABI, calibrating a bound, minting an identity, or weakening either relation when the target substrate is insufficient.
