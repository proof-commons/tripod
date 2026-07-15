# Simplicity Backend Package Plan

> **Status:** PARKED
> **Planned source directory:** `packages/simplicity`
> **Planned Cargo package:** `tripod-simplicity`
> **Planned Rust library name:** `simplicity`
> **Implementation phase:** Not on the active critical path; reconsider after
> the target-independent realization/compiler boundary and first Elements
> tapscript deployment path are demonstrated
> **Depends on future packages:** `tripod-compiler`, a typed
> Simplicity target compatibility contract (review provenance recorded;
> ADR-011), and the common linked-artifact roles proven by the first backend
> **Depends on decisions:**
> [D001](../decisions/001-typed-rust-is-normative.md),
> [D002](../decisions/002-target-independent-realization-layer.md),
> [D003](../decisions/003-multiple-backends-tapscript-first.md),
> [D004](../decisions/004-translation-validation-over-compiler-trust.md),
> [D005](../decisions/005-value-parametric-asset-rigid.md),
> [D006](../decisions/006-canonical-transaction-layout-abi.md)
> **Open research dependencies:** no Simplicity-specific research program is
> active yet; exact target, jet, resource, constructor, transaction, and
> deployment questions must be opened when the backend is reactivated
> **Authority:** Future target-specific implementation beneath the typed
> realization and compiler analysis
> **Machine-consumed planning document:** no

---

## 1. Purpose

The future `simplicity` package would provide a second target backend for the
same target-independent the attestation realization.

It would consume:

- a validated compiler analyzed program;
- compiler-approved target-independent proof alternatives;
- an exact typed Simplicity target definition;
- typed backend configuration;
- typed deployment parameters or relocations.

It would produce:

- typed relocatable Simplicity programs;
- object-constructor or commitment artifacts;
- target-specific proof selections;
- target-specific obligation placement;
- target-specific transaction-layout requirements;
- witness/program input requirements;
- resource formulas;
- relation-carrier provenance;
- backend-specific evidence requirements.

Its primary architectural purpose is not merely to produce another program
encoding. It is also to test that:

- semantic relations are genuinely target-independent;
- compiler analysis is not shaped around a stack machine;
- proof alternatives are not aliases for tapscript patterns;
- linker interfaces describe semantic constructor roles rather than only
  taptrees;
- transaction ABI concepts can support a target with different execution and
  witness structure;
- relation-indexed vectors can be reused across backends;
- target-specific evidence remains separate from semantic evidence.

The package is intentionally parked. It must not block the first Elements
tapscript backend or the first deployment evidence path.

---

## 2. Parking decision

The project designs the realization/compiler boundary for more than one
backend, but implements Elements tapscript first.

Simplicity remains parked until the first backend demonstrates the abstractions
the second backend would consume.

Parking means:

- no production Simplicity backend is promised in the current roadmap;
- no Simplicity package needs to exist in the Cargo workspace yet;
- no unused Simplicity dependencies are added;
- no placeholder target capabilities are treated as current facts;
- no release command claims Simplicity support;
- no generated Simplicity artifact is committed;
- no target-independent type may nevertheless assume tapscript;
- no speculative Simplicity architecture may delay the tapscript backend.

The active implementation remains:

```text
typed realization
    ↓
target-independent compiler analysis
    ↓
Elements target and tapscript backend
```

The future path remains:

```text
same typed realization
    ↓
same target-independent relation analysis
    ↓
typed Simplicity target and Simplicity backend
```

---

## 3. Why retain a second-backend plan

### 3.1 Behavioural conformance is not byte identity

The realization defines accepted semantic transitions, invariant preservation,
constructibility, lifecycle, and public observables.

It does not require one exact target program.

Two backends may use different:

- program languages;
- proof primitives;
- commitment structures;
- witness encodings;
- transaction layouts;
- resource units;
- compiler optimizations.

They conform only if both preserve the same target-independent semantic
relation and separately discharge their target/deployment evidence.

### 3.2 The second backend tests the abstraction boundary

A target-independent type can accidentally contain tapscript assumptions even
when its name appears generic.

Examples include:

- `Program` represented only as script bytes;
- `Layout` assuming one witness stack per transaction input;
- `Carrier` assuming a tapleaf;
- `Constructor` assuming an x-only key and taptree tweak;
- `Cost` assuming witness weight and crypto-budget units;
- `Proof` assuming stack inspection of transaction outputs;
- `Reference` assuming a scriptPubKey hash relocation.

A real second backend is the strongest practical test that those details
remain below the backend boundary.

### 3.3 Potential formal-assurance benefits

Simplicity's typed execution and formally specified semantics may eventually
provide stronger evidence for some target patterns, particularly:

- wide arithmetic;
- checked comparisons;
- structured hashes;
- commitment relations;
- target-program semantics;
- resource analysis.

Those benefits are prospective. They must not be claimed until the exact
selected target, jets, programs, compiler path, and deployment environment are
implemented and evidenced.

### 3.4 Backend diversity can reveal underspecification

If two independent backends disagree about:

- operation acceptance;
- state successor;
- recipient;
- event projection;
- public disclosure;
- lifecycle reachability;

the cause may be:

- a backend defect;
- a compiler lowering defect;
- an incomplete realization relation;
- an ambiguous proof alternative;
- a target capability mismatch.

Cross-backend comparison is therefore valuable additional evidence.

It does not replace comparison to the model/realization or target-native
deployment evidence.

---

## 4. Reactivation conditions

Simplicity work becomes active only after a roadmap update and review confirm
all of the following.

### 4.1 Semantic boundary exists

- `RealizationSpec` covers the operations selected for the first backend;
- semantic fact, expression, relation, observable, constructibility,
  lifecycle, and representation types are stable enough for another consumer;
- realization identity or internal schema policy is defined;
- declassification derives from typed dependencies;
- model conformance is established for the relevant scope.

### 4.2 Compiler boundary exists

- the compiler relation graph is target-independent in implementation;
- proof alternatives are distinct from target patterns;
- disclosure analysis is target-aware without being target-defined;
- target-capability requirements are typed;
- obligation placement has a backend-neutral requirement layer;
- layout requirements do not contain tapscript indexes;
- relation coverage definitions are reusable.

### 4.3 First backend has exercised hard seams

At minimum, the tapscript path should have exercised:

- one complete root-free operation;
- owner authorization;
- metadata-parameterized object construction;
- STATE succession;
- target arithmetic;
- public declassification;
- linking;
- transaction ABI;
- bundle-specific vectors.

A second backend introduced before these seams exist would co-design too many
unknowns at once.

### 4.4 Linked-artifact roles are understood

The project must know which linker concepts are actually backend-neutral, such
as:

- semantic constructor role;
- target program definition;
- symbol/reference;
- relocation or target reference;
- witness/program-input requirement;
- resource formula;
- source relation provenance.

Concrete tapscript types must not be generalized speculatively before this
review.

### 4.5 Exact target can be defined

The project must identify:

- the reviewed upstream Simplicity implementation (review provenance,
  not protocol identity; ADR-011);
- exact deployment network and activation state;
- leaf version or execution domain;
- compiler/library version;
- supported jets;
- target program commitment semantics;
- transaction-environment interface;
- resource/cost model;
- consensus and policy limits;
- deployment evidence requirements.

No backend begins from “latest Simplicity.”

### 4.6 Concrete project value exists

At least one of these should justify activation:

- stronger arithmetic assurance;
- production migration or redundancy need;
- material resource improvement;
- target deprecation risk;
- independent backend assurance requirement;
- external interoperability need;
- funded implementation capacity.

A second backend is not implemented solely to satisfy an architectural slogan.

---

## 5. Reactivation process

When the conditions above hold:

1. change this document's status from `PARKED` to `PROPOSED` or `ACTIVE`;
2. add a typed Simplicity target-contract plan or package;
3. open focused research notes for unresolved target behavior;
4. review D003 and update it if the backend boundary changed;
5. decide common versus target-specific linker artifact interfaces;
6. add the package to the roadmap;
7. define one limited pilot;
8. add the package to the Cargo workspace only when implementation begins;
9. establish target-native execution and evidence tooling;
10. produce a separate target/bundle/ABI/report identity chain.

The first implementation commit must not silently convert this parked plan into
a release promise.

---

## 6. First pilot

The first Simplicity pilot should be a semantic operation already implemented
and evidenced by tapscript.

The initial preferred pilot is:

```text
compact-ash
```

Reasons:

- no STATE constructor;
- no owner authorization;
- no wide division;
- no issuance;
- no distribution mapping;
- no maturity;
- no confidential-to-public transition required for the basic explicit/public
  profile;
- bounded ownerless aggregation;
- permissionless construction;
- exact closed-asset conservation;
- complete relation-indexed vector set should already exist.

The pilot must reuse:

- the same architecture operation ID;
- the same realization relations;
- the same semantic vector definitions;
- the same expected semantic successor and public projection.

It must produce separate:

- Simplicity target identity;
- backend configuration identity;
- emitted program/bundle identity;
- transaction ABI identity;
- resource report;
- target-native execution report;
- relation-coverage report;
- substrate evidence.

A successful pilot does not automatically activate all remaining operations.

---

## 7. Planned package responsibilities after reactivation

If activated, the `simplicity` package would own:

- target-specific program construction;
- lowering compiler-approved proof plans into Simplicity;
- typed target-program representation;
- target type checking;
- jet selection;
- target commitment identities;
- target-specific object-constructor proof patterns;
- target-specific obligation placement;
- target-specific layout lowering;
- witness/program-input schemas;
- resource formulas;
- relocatable program artifacts;
- relation-carrier provenance;
- backend-specific tests and reports.

It would not own:

- architecture registries;
- realization semantics;
- compiler relation identity;
- declassification policy;
- final deployment profile;
- independent event/query/accounting implementations;
- tapscript behavior;
- a claim that its target semantics prove the compiler.

---

## 8. Normative typed inputs

A future backend would consume:

```text
compiler::AnalyzedProgram
compiler-approved proof alternatives
typed Simplicity target
typed Simplicity backend configuration
typed deployment parameters or relocations
```

It would not consume:

- architecture JSON/TOML;
- declassification JSON;
- planning Markdown;
- model source;
- tapscript bytes;
- tapscript disassembly;
- a linked tapscript bundle as semantic input.

Cross-backend tests may compare typed semantic results and report identities,
not use one backend's output as the other's semantics.

---

## 9. Target-independent constraints the current toolchain must preserve

While Simplicity is parked, active package design must preserve these
constraints.

### 9.1 Realization

`RealizationSpec` must not contain:

- opcode names;
- stack positions;
- tapleaf identities;
- taptree structure;
- control blocks;
- tapscript cost formulas;
- Elements-specific witness order.

### 9.2 Compiler

Compiler-core types must distinguish:

- semantic relation;
- proof alternative;
- selected target proof;
- target capability;
- target carrier requirement;
- target-specific layout result.

Compiler relation identity must not depend on tapscript.

### 9.3 Linker

Shared linker roles must not be falsely defined as:

- scriptPubKey-only symbols;
- tapleaf-only programs;
- taptree-only constructors;
- control-block-only witness data.

The tapscript linker path may remain concrete. Any common abstraction should be
named according to semantic role.

### 9.4 Transaction ABI

D006 permits one target-specific ABI per bundle.

The transaction package must not claim tapscript's witness/control layout is
universal.

### 9.5 Vectors

Semantic vectors must remain separable from:

- target transaction bytes;
- witness serialization;
- target program identity;
- target resource units.

### 9.6 Release

Deployment profiles and release records must bind exact target and backend
identities.

A tapscript report cannot be reused for Simplicity.

---

## 10. Target and capability plan after reactivation

A complete Simplicity backend requires a typed target definition analogous in
role—not necessarily schema—to `target-elements`.

It must bind:

- upstream source and implementation;
- target network/deployment identity;
- activation;
- execution domain;
- supported jets;
- base combinator semantics relied upon;
- transaction-environment interface;
- program commitment identities;
- witness/program input model;
- resource/cost model;
- consensus and policy constraints;
- target evidence requirements.

The target package should distinguish:

```text
target primitive available
approved complete attestation-contract proof pattern available
deployment evidence verified
```

Those remain three different states.

The target definition must not derive capability claims by parsing this plan.

---

## 11. Program representation

The future backend should use typed Simplicity program structures from the
selected reviewed implementation (pinned via `Cargo.lock` when a library)
or a narrow first-party typed wrapper.

It must not lower through tapscript-shaped intermediate instructions.

The program representation should preserve:

- source realization relation IDs;
- selected proof alternatives;
- target types;
- program commitment identities;
- target requirement provenance;
- witness/program input requirements;
- resource formulas;
- target errors.

The exact representation remains deferred until the target is selected.

---

## 12. Proof and jet selection

The backend may use:

- primitive combinators;
- approved jets;
- structured target programs;
- formally specified target operations.

Selection must be:

- semantics-preserving;
- target-supported;
- deterministic;
- resource-aware;
- source-provenanced;
- identity-bound.

A jet is an implementation of a semantic target expression, not the semantic
relation itself.

If jet and unjetted forms are intended to be equivalent, evidence must bind the
exact target semantics and implementation.

The compiler/backend must reject a required relation for which no valid target
proof exists.

---

## 13. Constructor and state continuity

The future target may use constructor or program-commitment mechanisms
different from tapscript's taptree/tweak model.

The backend must nevertheless establish the same semantic requirements:

- predecessor object authentication;
- metadata authenticity;
- static program continuity;
- successor metadata correctness;
- root succession or termination;
- no alternate escape program;
- closed-asset object closure;
- canonical public observables.

Do not force the tapscript STATE constructor design onto Simplicity.

The first STATE-spending Simplicity operation should be selected only after the
root-free pilot validates the backend boundary.

---

## 14. Representation policy

D005 remains applicable.

The backend may support different target proof methods for:

- explicit values;
- private commitments;
- public commitments;
- commitment equality;
- confidential value conservation;
- authenticated opening.

It must preserve:

- explicit/rigid closed protocol asset classification under the current
  deployment policy unless superseded;
- permissionless constructibility;
- lifecycle exits;
- disclosure provenance;
- separate safety and minimality evidence.

A stronger target opening or arithmetic proof can expand supported
representations without changing semantic value.

Any proposal for confidential closed-asset identity requires a new or
superseding decision under D005.

---

## 15. Transaction and witness ABI

The Simplicity backend receives its own target-specific ABI under D006.

It may differ from tapscript in:

- program selection;
- witness/program input structure;
- transaction layout;
- coordinator strategy;
- constructor proof;
- control/commitment data;
- resource formulas.

The ABI must remain:

- deterministic;
- bundle-bound;
- target-bound;
- typed;
- canonically publishable;
- usable by clients, vectors, and calibration;
- free of secret values.

The backend may reuse target-independent layout requirements while choosing a
different concrete arrangement.

---

## 16. Linker integration

The future linker path must support target-specific program and constructor
artifacts.

Potential common roles include:

- program symbol;
- constructor symbol;
- target reference;
- deployment relocation;
- witness/program-input requirement;
- resource formula;
- relation carrier;
- bundle manifest.

Concrete Simplicity linking may involve:

- program commitment roots;
- target environment commitments;
- target-specific constructor references;
- target-specific deployment constants.

It must not be encoded artificially as a tapscript taptree.

The first Simplicity pilot should produce a separate linked-bundle identity.

---

## 17. Evidence model

D004 applies independently to the Simplicity backend.

### 17.1 Required evidence

At minimum:

- target-independent relation census equality;
- selected proof-plan report;
- relation carrier mapping;
- positive vectors;
- focused rejecting mutations;
- target-native program execution;
- public semantic projection comparison;
- transaction ABI vectors;
- resource report;
- target substrate/deployment reports;
- deterministic bundle reproduction.

### 17.2 Shared semantic vectors

Reuse semantic fixture and mutation definitions where they are genuinely
target-independent.

Examples for `compact-ash`:

- too few ASH inputs;
- wrong asset;
- wrong object kind;
- wrong output count;
- wrong output value;
- closed-asset escape;
- hidden authorization;
- malformed sponsor region.

Concrete transaction/program materialization remains backend-specific.

### 17.3 Cross-backend differential

Once both backends implement an operation, compare:

- semantic verdict;
- successor abstract state;
- canonical deltas;
- public observables;
- event projections;
- disclosure result;
- lifecycle behavior.

Do not require equality of:

- target program bytes;
- program commitments;
- transaction bytes;
- witness representation;
- raw cost metrics.

### 17.4 Formal evidence scope

If Simplicity or its jets provide formal semantic evidence, reports must state
the exact proven scope.

For example:

```text
this jet implements this target expression
```

does not automatically prove:

- compiler selected the right expression;
- object constructor is correct;
- linker committed the intended program;
- transaction ABI supplied the right witness;
- target deployment activated the intended semantics.

### 17.5 Independent observers

The same independently implemented:

- event projector;
- attestation query implementation;
- receipt-accounting auditor

may observe both deployments if their public wire/interface facts are the same
and their evidence is correctly bound to each target context.

Their reports remain distinct from backend evidence.

---

## 18. Resource model

Simplicity resource units may differ materially from tapscript.

The future target plan must define:

- target program cost model;
- jet costs;
- program size;
- witness/program input size;
- transaction weight interaction;
- consensus limits;
- policy limits;
- target-specific calibration dimensions.

Do not translate tapscript metrics mechanically into Simplicity metrics.

Shared deployment bounds may need separate calibration for each backend.

A deployment profile binds the calibrated values for the selected target
bundle.

---

## 19. Determinism and identity

The future backend must produce separate deterministic identities for:

- exact Simplicity target definition;
- deployment instance;
- backend configuration;
- selected proof plan;
- relocatable program set;
- linked bundle;
- transaction ABI;
- vector set;
- execution/resource reports.

The realization and semantic relation identities remain backend-independent.

Concrete program identity may use target-native commitment schemes, but the
release bundle must bind them canonically.

No target identity may depend on:

- local checkout path;
- current time;
- host environment;
- mutable network endpoint;
- nondeterministic compiler search.

---

## 20. Prohibited consequences

Reactivating Simplicity must not:

- change protocol semantics to fit target capabilities;
- redefine architecture IDs;
- replace the target-independent realization with Simplicity combinators;
- make Simplicity jets semantic relation IDs;
- reuse tapscript bundle evidence;
- reuse tapscript target identity;
- claim formal compiler correctness from target-language formal semantics;
- weaken permissionless constructibility;
- introduce confidential closed-asset identity without a new decision;
- bypass the transaction ABI;
- omit relation-level vectors;
- make the initial tapscript deployment dependent on Simplicity;
- parse planning Markdown as target configuration.

While parked, the project must not:

- add empty generated Simplicity files;
- publish placeholder bundle hashes;
- advertise a supported backend;
- add dependencies solely for appearance;
- write detailed source claims without an exact pin.

---

## 21. Initial package API after reactivation

A future API may conceptually resemble:

```rust
pub fn emit(
    plan: &compiler::TargetCompilationPlan,
    target: &SimplicityTarget,
    configuration: &SimplicityConfiguration,
) -> Result<RelocatableSimplicityBundle, SimplicityError>;
```

> Illustrative API; not frozen and not an active implementation commitment.

The output should have roles corresponding to:

- programs;
- constructors;
- target references;
- deployment relocations;
- obligation placement;
- layout;
- witness/program input schema;
- resource formulas;
- relation carriers.

The concrete types should follow the selected Simplicity implementation rather
than imitate tapscript bytes.

---

## 22. Likely package structure after reactivation

A possible future structure is:

```text
simplicity/src/
├── lib.rs
├── error.rs
├── configuration.rs
├── identity.rs
├── target.rs
├── program.rs
├── emit.rs
├── proof.rs
├── jets.rs
├── constructor.rs
├── layout.rs
├── witness.rs
├── relocation.rs
├── resources.rs
└── tests/
```

> Illustrative only. Do not create this skeleton while the package is parked.

A separate typed target package may be preferable if target facts are used by
transaction, vectors, and release independently of backend emission.

---

## 23. Milestones after reactivation

### S1 — Target contract and capability model

Deliver:

- reviewed upstream target (review provenance recorded);
- activation/network identity;
- supported primitives and jets;
- resource model;
- evidence requirements.

### S2 — Minimal backend adapter

Deliver:

- typed program construction;
- compiler proof-plan input;
- deterministic identity;
- no tapscript dependency.

### S3 — `compact-ash` emission

Deliver:

- all semantic relations;
- object constructor;
- permissionless construction;
- target-specific layout/ABI requirements;
- relation carriers.

### S4 — Target transaction integration

Deliver:

- linked program;
- canonical transaction/witness ABI;
- target-native execution;
- resource measurement.

### S5 — Shared and cross-backend vectors

Deliver:

- semantic vector reuse;
- target-specific materialization;
- relation coverage;
- comparison with tapscript semantic results.

### S6 — Backend-boundary review

Review:

- leaked tapscript assumptions;
- common linker roles;
- common transaction roles;
- common report schemas;
- compiler target neutrality.

### S7 — Next operation selection

Choose based on the assurance goal:

- live transfer for representation proof comparison;
- announce maturity for STATE constructor comparison;
- wide arithmetic operation for formal arithmetic comparison.

Do not commit the complete operation sequence before the pilot review.

---

## 24. Exit criteria for the first pilot

The first Simplicity pilot is complete only when:

- [ ] the backend has been explicitly reactivated;
- [ ] an exact typed Simplicity target contract exists;
- [ ] the package is in the workspace;
- [ ] no tapscript package dependency exists;
- [ ] the same `compact-ash` realization relations are consumed;
- [ ] relation IDs remain unchanged;
- [ ] every selected proof is target-supported;
- [ ] every relation has a Simplicity carrier;
- [ ] permissionless construction requires no private owner/operator witness;
- [ ] closed `U` identity is enforced under current representation policy;
- [ ] exact ownerless value conservation is enforced;
- [ ] exactly one ASH successor is enforced;
- [ ] sponsor isolation is enforced where supported;
- [ ] target-specific layout and witness/program input ABI are typed;
- [ ] the program links deterministically;
- [ ] valid target transactions execute;
- [ ] focused mutations reject;
- [ ] public semantic result matches the model and tapscript result;
- [ ] resource measurements fit target limits;
- [ ] target and bundle identities verify;
- [ ] backend-specific reports are complete;
- [ ] cross-backend comparison does not replace target-native evidence;
- [ ] debug/release workspace checks pass;
- [ ] the checkout remains clean.

---

## 25. Non-goals while parked

The current roadmap does not include:

- creating `packages/simplicity`;
- selecting a Simplicity library;
- reviewing a Simplicity implementation;
- emitting Simplicity programs;
- building Simplicity transactions;
- defining production jets;
- calibrating Simplicity bounds;
- publishing a Simplicity ABI;
- making release depend on Simplicity;
- claiming backend diversity has been achieved.

The active responsibility is only to keep the target-independent boundary
honest enough that future work remains possible.

---

## 26. Open questions for reactivation

These questions should not be answered speculatively now.

### 26.1 Exact target and implementation

Which reviewed implementation, execution environment, and deployment
profile will be used?

### 26.2 Target package ownership

Will Simplicity target facts live:

- in the backend package;
- in a separate `target-simplicity` package;
- in an upstream typed library adapter?

### 26.3 Compiler adapter

How will compiler abstract capabilities map to target expressions and jets?

### 26.4 Constructor continuity

How will object and STATE metadata bind to target programs and successors?

### 26.5 Transaction environment

Which transaction fields and proofs are accessible to programs?

### 26.6 Witness model

How are target program inputs formed, authenticated, and exposed in a public
ABI?

### 26.7 Linking

Which program/constructor references are resolved at compile time, link time,
or execution time?

### 26.8 Confidential values

Which value/asset commitment relations are directly supported, and what
constructibility evidence is needed?

### 26.9 Resource calibration

Which target units and whole-transaction constraints determine protocol
bounds?

### 26.10 Formal evidence

Which target or jet correctness claims are formally established, and where
does the remaining compiler/linker/transaction trust boundary lie?

---

## 27. Risks

### 27.1 Stale planning assumptions

The target may evolve before reactivation.

Mitigation:

- keep this plan architectural and short;
- source-pin only at reactivation;
- avoid current capability promises.

### 27.2 Tapscript leakage discovered late

A second backend may expose assumptions embedded in compiler or linker types.

Mitigation:

- preserve explicit target boundaries now;
- perform a backend-boundary audit as an early milestone;
- permit deliberate identity/schema migration.

### 27.3 Duplicate infrastructure

A second backend may duplicate target execution, transaction, linker, and
report code.

Mitigation:

- share only demonstrated target-neutral roles;
- keep semantic vectors common;
- avoid forcing one target's representation onto another.

### 27.4 Formal-assurance overclaim

Simplicity's formal foundations may be misrepresented as proving the complete
toolchain.

Mitigation:

- evidence reports state exact theorem scope;
- D004 remains in force;
- compiler, linker, transaction, target deployment, and independent observer
  boundaries remain separate.

### 27.5 Roadmap distraction

A promising second backend may divert effort before the first deployment is
complete.

Mitigation:

- explicit reactivation conditions;
- roadmap review;
- concrete project-value requirement;
- tapscript remains first.

### 27.6 Lowest-common-denominator compiler

Attempts to support both targets may weaken proof planning or avoid useful
target-specific features.

Mitigation:

- semantic relation remains strong;
- proof alternatives are target-specific;
- unsupported targets fail;
- no requirement for identical layouts or cost models.

### 27.7 Identity migration

A second backend may require changes to supposedly target-independent
identities.

Mitigation:

- do not freeze realization/compiler identities prematurely;
- document canonical projections;
- use explicit schema/algorithm versions;
- retain migration records.

---

## 28. Supersession conditions

This plan may be superseded if:

- Simplicity becomes the primary backend;
- the project permanently rejects a second backend;
- a different second target is selected;
- a unified formally verified target architecture replaces both backend plans;
- target-independent compiler boundaries are redesigned.

A superseding decision must address:

- realization and compiler identity compatibility;
- package dependency migration;
- linker and transaction ABI effects;
- evidence/report migration;
- deployment profile changes;
- target-specific release status;
- effect on the existing tapscript backend.

Parking may continue indefinitely without superseding D003. The decision is
about preserving a real backend boundary, not promising a delivery date.

---

## 29. References

### Related decisions

- [D001: Typed Rust Is Normative](../decisions/001-typed-rust-is-normative.md)
- [D002: Introduce a Target-Independent Realization Layer](../decisions/002-target-independent-realization-layer.md)
- [D003: Design for Multiple Backends and Implement Elements Tapscript First](../decisions/003-multiple-backends-tapscript-first.md)
- [D004: Use Per-Bundle Translation Validation Instead of Initially Trusting the Compiler](../decisions/004-translation-validation-over-compiler-trust.md)
- [D005: Permit Value-Representation Latitude While Keeping Closed Asset Identity Rigid](../decisions/005-value-parametric-asset-rigid.md)
- [D006: Generate a Canonical Transaction and Witness ABI for Each Backend Bundle](../decisions/006-canonical-transaction-layout-abi.md)

### Package plans

- [`compiler.md`](compiler.md)
- [`target-elements.md`](target-elements.md)
- [`tapscript.md`](tapscript.md)
- [`linker.md`](linker.md)
- [`transaction.md`](transaction.md)
- [`vectors.md`](vectors.md)
- [`release.md`](release.md)

### Roadmap and architecture

- [`../toolchain-architecture.md`](../toolchain-architecture.md)
- [`../roadmap.md`](../roadmap.md)

---

## 30. Definition of done

This parked package plan has fulfilled its present purpose when the active
realization, compiler, linker, transaction, vector, and release designs keep
tapscript-specific details below an explicit backend boundary; no production
claim or dependency is created prematurely; the reactivation conditions are
clear; and a future Simplicity implementation can begin with one exact target
and one existing semantic pilot rather than requiring a rewrite of the
target-independent protocol relation.

---

## 31. One-line package contract

> Keep Simplicity as a parked second backend that will consume the same typed
> realization relations and compiler analysis, emit its own target programs,
> constructors, ABI, resource formulas, and evidence, and serve as a practical
> audit of backend neutrality—without blocking the first tapscript deployment,
> leaking speculative target concepts into semantic types, or claiming formal
> compiler correctness before the complete translation and deployment boundary
> is proved.
