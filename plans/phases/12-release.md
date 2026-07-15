# Phase 12 — Calibration, Evidence, and Release · `phase:roadmap:release`

> **Status:** Planned
> **Entry:** (`gate:phase4:exit`) through (`gate:phase11:exit`)
> **Packages:** [`vectors`](../packages/vectors.md),
> [`release`](../packages/release.md)
> **Final subject:** one exact deployment bundle and profile

## Goal · `sec:phase12:goal`

Assemble complete, identity-bound, independently reviewed deployment evidence;
calibrate every finite bound against valid complete transactions; construct the
final architecture-owned deployment profile; and publish one deterministic
release only after every required gate passes.

## Entry conditions · `sec:phase12:entry`

- all approved operations implemented;
- complete realization and compiler relation scope;
- exact production target/deployment selected;
- final backend/linker/transaction interfaces available;
- independent observer implementations available;
- no unresolved release-blocking research;
- no candidate/prototype artifact in the final scope.

## Calibration · `sec:phase12:calibration`

For every architecture bound requiring deployment calibration:

1. derive every affected operation family;
2. select candidate assignment;
3. link candidate bundle;
4. derive candidate ABI;
5. construct valid objective-specific worst-case transactions;
6. execute and measure;
7. choose the next candidate deterministically;
8. select final values;
9. relink the final bundle;
10. regenerate the final ABI;
11. regenerate and remeasure every affected transaction.

Require exact bundle/ABI/report identity binding.

Draft defaults are never final evidence.

## Required artifacts · `sec:phase12:artifacts`

Final release assets include typed/canonical forms of:

- architecture publications;
- realization identity/publication where adopted;
- compiler configuration and analysis identity;
- target definition and deployment binding;
- linked program bundle;
- transaction ABI;
- canonical wire and transaction vectors;
- calibration report;
- relation coverage;
- model/property reports;
- target dependency reports;
- script-integration report;
- independent observer reports;
- deployment profile;
- release manifest.

The exact file census is release-policy owned.

## Evidence census · `sec:phase12:evidence`

Require separate passed reports for:

- model unit tests;
- property tests;
- realization/model conformance;
- compiler analysis;
- backend patterns;
- exact linked-bundle execution;
- relation coverage;
- representation safety;
- representation minimality where claimed;
- resource calibration;
- target dependencies;
- script integration;
- independent events;
- independent query;
- independent accounting.

No required report may be:

```text
missing
failed
incomplete
unsupported
skipped
stale
zero-hash
infrastructure-error
```

No general waiver policy exists.

## Independent observers · `rule:phase12:observers`

The event, query, and accounting claims remain separate.

Each candidate report records:

- implementation identity;
- source/tool version;
- shared dependencies;
- context;
- report identity.

A wrapper around the model/reference implementation does not satisfy the
independence requirement.

## Deployment profile · `sec:phase12:profile`

Construct the architecture-owned final profile with:

- supported schema;
- final status;
- architecture semantic hash;
- network/genesis;
- final script limits;
- calibrated bounds;
- verified dependency evidence;
- artifact hashes;
- separate test report hashes.

Run typed deployment-release validation.

Compute and verify the domain-separated profile hash.

New compiler-era reports not represented directly in the current profile
schema remain bound by the release manifest until a reviewed profile revision
adds typed fields.

## Publication · `sec:phase12:publication`

Publication:

1. assembles in a unique staging directory;
2. writes the exact canonical asset census;
3. rereads and verifies every byte and hash;
4. validates release manifest and deployment profile;
5. checks for secrets and unexpected files;
6. publishes atomically where practical;
7. returns a publication receipt.

The non-writing checker independently recomputes and compares the release.

## Reproducibility · `sec:phase12:reproducibility`

Two clean release assemblies with identical explicit inputs must produce
byte-identical:

- manifests;
- profiles;
- generated publications;
- linked bundles;
- ABIs;
- canonical reports;
- release documents;
- canonical archive or directory representation.

Release date and source date are explicit.

## Final evidence · `sec:phase12:final-evidence`

The release record contains or links:

- source revision;
- Attestation and architecture bindings;
- realization/compiler/target/bundle/ABI identities;
- calibrated values;
- report identities;
- deployment-profile identity;
- publication asset hashes;
- reproduction command and result;
- named residual assumptions.

## Exit gate · `gate:phase12:exit`

Phase 12 exits only when:

- full approved operation scope is complete;
- every semantic relation has reachable positive and negative bundle evidence;
- exact production target and deployment instance are bound;
- every target dependency has required verified evidence;
- every calibrated bound is final, complete, and remeasured;
- runtime and profile bounds agree;
- final linked bundle and ABI identities verify;
- architecture publications and document welds are current;
- independent event, query, and accounting reports pass separately;
- architecture deployment validation passes;
- profile and release identities verify;
- publication is deterministic, secret-free, and atomically staged;
- non-writing release check passes;
- all repository lanes pass;
- no tracked file changes during checks;
- the final release record is committed or published under explicit policy.
