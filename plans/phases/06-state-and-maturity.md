# Phase 6 — STATE Constructor and Maturity Announcement · `phase:roadmap:state-maturity`

> **Status:** Active — opened by the owner 2026-08-27 as the roadmap's
> current phase when Phase 5 exited; both entry conditions hold — the
> amended phase-5 exit gate is met on its countersigned assessment, and
> the STATE-constructor research decision is accepted. No execution
> guide is chartered yet: the Guide-14 drafting decision is the owner's
> and remains open.
> **Entry:** (`gate:phase5:exit`) and accepted STATE-constructor decision
> **Packages:** tapscript, linker, transaction, vectors
> **Operation:** `announce-maturity`

## Goal · `sec:phase6:goal`

Integrate authenticated metadata-dependent STATE succession in the smallest
STATE-changing operation.

The implementation must preserve STATE metadata and static code continuity
under the accepted constructor design.

## Deliverables · `sec:phase6:deliverables`

### STATE constructor

Implement the accepted constructor recipe:

- canonical STATE metadata schema;
- deterministic metadata commitment;
- linked static operation-program identity;
- predecessor constructor authentication;
- successor constructor reconstruction;
- internal-key policy;
- metadata path unspendability;
- target control-path recipe;
- constructor resource formula.

### Operation relation

Implement `announce-maturity`:

- canonical STATE predecessor;
- maturity currently unannounced;
- announced cycle within minimum and maximum leads;
- successor maturity announced;
- all economic state fields unchanged;
- operator authorization;
- no RESV use;
- optional isolated sponsor flow;
- STATE succession projection.

### ABI

Define:

- STATE input slot;
- STATE successor slot;
- operator witness role;
- constructor/static-root metadata;
- sponsor region;
- target transaction version/sequence constraints.

## Required vectors · `sec:phase6:vectors`

Positive:

- minimum valid lead;
- maximum valid lead;
- valid operator;
- sponsorless/sponsored;
- exact successor metadata.

Negative:

- below minimum;
- above maximum;
- second announcement;
- sealed predecessor;
- wrong operator;
- wrong STATE asset or amount;
- wrong predecessor metadata;
- changed unaffected state field;
- wrong successor metadata;
- successor under another static root;
- extra/missing operation leaf;
- wrong internal key;
- wrong parity/control path;
- metadata-leaf spend;
- unexpected RESV;
- missing STATE successor;
- stale constructor from another bundle.

## History evidence · `sec:phase6:history`

Extend constructor and root-history fixtures to reject:

- intermediate constructor substitution;
- final cursor restored after an invalid intermediate STATE edge;
- old-bundle predecessor/new-bundle successor without an approved migration;
- post-sealing STATE revival.

## Resource evidence · `sec:phase6:resources`

Measure complete announce-maturity transactions with:

- representative and maximum metadata;
- deepest control path;
- operator signature;
- maximum sponsor candidate;
- constructor verification.

Success here validates the constructor mechanism for this operation, not
resource feasibility of every later STATE operation.

## Exit gate · `gate:phase6:exit`

Phase 6 exits when:

- accepted constructor research is implemented exactly;
- metadata encoding is canonical;
- predecessor and successor share one authenticated static code identity;
- no key-path or metadata path bypass remains;
- announce-maturity matches the model relation;
- all wrong-code-subtree vectors reject;
- root-history continuity tests pass;
- target and policy resources fit;
- bundle/ABI/report identities are deterministic;
- relation coverage is complete and the repository remains clean.
