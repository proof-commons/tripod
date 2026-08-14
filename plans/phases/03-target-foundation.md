# Phase 3 — Elements Target and Foundational Prototypes · `phase:roadmap:target-foundation`

> **Status:** Active — typed target contract, capability adapter, typed
> instruction core, and development target-native primitive evidence
> implemented; the STATE-constructor, wide-arithmetic, and public
> declassification prototypes remain open; prototype-driven
> **Entry:** (`gate:phase2:exit`)
> **Packages:** [`target-elements`](../packages/target-elements.md),
> [`tapscript`](../packages/tapscript.md)
> **Research:** STATE constructor, wide arithmetic, public declassification

## Goal · `sec:phase3:goal`

Replace target assumptions with one typed Liquid/Elements compatibility
contract and validate the backend’s load-bearing primitives before production
operation interfaces freeze.

## Entry conditions · `sec:phase3:entry`

- compiler analysis exists for both pilots;
- target capability requirements are typed;
- target-specific concerns remain below compiler core;
- development target environment can be created hermetically;
- target-source/node provenance policy follows
  (`[ADR011-rule:toolchain:target-compatibility]`).

Isolated research may begin earlier, but production APIs wait for the Phase-2
boundary.

## Deliverables · `sec:phase3:deliverables`

### Typed target contract

Create `target-elements` with:

- target-definition schema and identity;
- development deployment instance;
- execution domain and leaf version;
- backend-used opcode discriminants;
- stack and failure contracts;
- asset/value/nonce encodings;
- field-specific byte order;
- sighash capabilities;
- relative timelock behavior;
- CT and issuance capabilities;
- consensus and policy resource interfaces;
- target evidence requirement registry.

Node implementation revisions remain test provenance, not target identity.

### Tapscript foundation

Create enough backend infrastructure for prototypes:

- typed instruction builder;
- deterministic serialization;
- stack/altstack contracts;
- branch validation;
- target capability checks;
- explicit/confidential representation guards;
- basic introspection;
- signature/timelock pattern interfaces;
- typed symbols and relocations;
- target resource formulas.

### STATE constructor prototype

The prototype must test:

- canonical metadata encoding;
- predecessor constructor authentication;
- successor reconstruction;
- static code continuity;
- one authenticated static root reused across the transition;
- internal-key policy;
- x-only/compressed-point bridge;
- hash/tweak totality policy;
- metadata-leaf unspendability;
- wrong subtree/key/parity/schema rejection;
- target resources.

Result handoff:
[STATE constructor research](../research/state-constructor.md).

### Wide arithmetic prototype

The prototype must test exact:

```text
q = floor(a*b/d)
```

using a target proof such as quotient/remainder multi-limb verification.

Required:

- checked domains;
- exact wide equality;
- remainder bound;
- malformed limb/carry rejection;
- quotient below/above rejection;
- stack contract;
- target-native vectors;
- operation-level feasibility measurements.

Result handoff:
[wide arithmetic research](../research/wide-arithmetic.md).

### Public declassification prototype

Compare:

- explicit boundary output;
- public commitment with authenticated opening;
- owner-authorized normalization;
- explicit-only initial policy.

The prototype must demonstrate permissionless future use and residual blinding
closure before direct confidential burn/redemption support is accepted.

Result handoff:
[public declassification research](../research/public-declassification.md).

## Evidence · `sec:phase3:evidence`

### Target contract

- opcode and encoding uniqueness;
- stack/failure contract tests;
- capability dependency closure;
- source/review provenance;
- deployment binding;
- deterministic target identity;
- target-native capability tests.

### Backend foundation

- instruction-byte vectors;
- stack and branch tests;
- representation guard vectors;
- signature/timelock boundary vectors;
- resource formula checks;
- deterministic emission.

### Research

Each accepted prototype includes:

- exact target;
- exact program identity;
- positive/negative vectors;
- predicted/observed resources;
- reproducible report;
- accepted decision or explicit rejection.

## Non-goals · `sec:phase3:non-goals`

Phase 3 does not claim:

- a complete protocol backend;
- production activation from regtest alone;
- a verified compiler;
- direct private boundary support without accepted research;
- final calibrated bounds.

## Exit gate · `gate:phase3:exit`

Phase 3 exits when:

- the typed target contract covers every primitive used by the first backend;
- development target binding and target-native tests pass;
- tapscript instruction and stack infrastructure is deterministic;
- unknown encodings and unsupported capabilities fail closed;
- STATE constructor and wide arithmetic produce accepted decisions or explicit
  target rejection;
- public declassification produces an accepted initial policy or explicit
  deferral;
- prototype code cannot enter release output silently;
- package contracts and research notes reflect results;
- workspace checks remain green and clean.
