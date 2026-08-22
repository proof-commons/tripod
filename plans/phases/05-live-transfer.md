# Phase 5 — Live Receipt Transfer · `phase:roadmap:live-transfer`

> **Status:** Active — the entry gate is satisfied: Phase 4 exited
> 2026-08-21, recorded in the backlog's section 2.8. The Guide-13
> batch is chartered; its preflight register gates implementation.
> **Entry:** (`gate:phase4:exit`)
> **Packages:** realization, compiler, tapscript, linker, transaction, vectors
> **Decision:** D005 value representation

## Goal · `sec:phase5:goal`

Validate:

- every-owner authorization;
- live-class closure;
- closed-asset output closure;
- split/merge conservation;
- explicit and confidential value proof alternatives;
- sponsor isolation;
- separate safety and disclosure-minimality evidence.

## Deliverables · `sec:phase5:deliverables`

### Constructors

Link a live-receipt constructor binding:

- explicit `U` asset;
- owner metadata;
- live class;
- selected value representation;
- transfer operation programs.

### Programs

Emit:

- local live receipt recognition;
- owner authorization on every input;
- canonical coordinator;
- complete input/output family ranges;
- live output closure;
- exact aggregate value conservation;
- sponsor isolation;
- representation-specific target proof.

### ABI

Support typed requests for:

- receipt inputs;
- destination owners and values;
- explicit or supported confidential representation;
- optional sponsor.

All signature-committed outputs are finalized before signing.

## Required safety vectors · `sec:phase5:safety`

- empty input/output;
- above input/output bounds;
- wrong or missing owner;
- one omitted owner in a multi-owner transfer;
- time-locked input;
- time-locked output;
- ASH or other undeclared output;
- wrong explicit asset;
- confidential asset commitment;
- unclassified closed `U`;
- aggregate value mismatch;
- duplicate output claim;
- sponsor overlap;
- output mutation after signing;
- wrong constructor metadata;
- transfer/burn program mixture.

## Required minimality vectors · `sec:phase5:minimality`

Compare semantically equivalent:

- explicit one-to-one transfer;
- explicit split;
- explicit merge;
- confidential-value one-to-one transfer;
- confidential-value split/merge where supported;
- confidential sponsor value where supported.

Require:

- identical semantic value relation;
- identical owner/class result;
- identical public protocol projection;
- target acceptance;
- no confidential closed-asset identity.

## Lifecycle · `sec:phase5:lifecycle`

A private live receipt is not release-complete until it retains supported paths
to:

```text
transfer
burn
redeem
```

Phase 5 records future lifecycle obligations explicitly. It does not overclaim
direct private burn or redemption support.

## Resource evidence · `sec:phase5:resources`

Measure:

- one input/one output;
- maximum candidate families;
- multi-owner signatures;
- explicit and confidential values;
- sponsorless/sponsored;
- largest proof and witness forms.

## Exit gate · `gate:phase5:exit`

Phase 5 exits when:

- every owner authorizes the finalized output set;
- exact live-class closure holds;
- all closed-asset-capable outputs are classified;
- explicit transfer passes complete safety evidence;
- confidential-value transfer passes where claimed;
- safety and minimality reports remain distinct;
- lifecycle incompleteness is explicit;
- mixed-program vectors reject;
- linked bundle and ABI are deterministic;
- resource reports pass;
- relation coverage is complete and the tree remains clean.
