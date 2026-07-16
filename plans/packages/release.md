# Release and Deployment Profile · `pkg:release:contract`

> **Status:** Planned
> **Phase:** [Phase 12](../phases/12-release.md)
> **Package:** `tripod-release`
> **Library:** `release`
> **Direct dependencies:** `architecture`, `realization`, `compiler`,
> `target-elements`, `linker`, `transaction`, `vectors`, `artifacts`
> **Decision:** [D004](../decisions/004-translation-validation.md)

## Purpose · `sec:release:purpose`

`release` is the final fail-closed assembly boundary for one deployment
candidate.

It validates and binds:

- architecture release;
- complete realization;
- compiler analysis;
- target and deployment instance;
- final linked bundle;
- final transaction ABI;
- calibrated bounds;
- generated publications;
- model and property reports;
- relation-indexed target evidence;
- target dependency reports;
- independent event, query, and accounting reports;
- final deployment profile;
- deterministic publication assets.

It does not define or repair semantics.

## Dependencies · `sec:release:dependencies`

Broad direct dependencies are intentional because release assembles all prior
typed boundaries.

No package above may depend on `release`.

The release package should not depend directly on `tapscript` when final target
artifacts are exposed through `linker`.

## Typed inputs · `sec:release:inputs`

Release consumes only validated typed values or explicitly parsed and validated
external report envelopes:

- architecture;
- realization;
- analyzed compiler plan;
- exact target/deployment;
- final linked bundle;
- final ABI;
- calibration;
- evidence set;
- publication asset set;
- explicit source revision and release date;
- typed release policy.

Generated architecture files are compared with typed expected values rather
than used as semantic inputs.

## Release states · `rule:release:states`

Use distinct types or validated states for:

```text
ReleaseInputs
ReleaseCandidate
ValidatedRelease
PublishedReleaseReceipt
```

Only `ValidatedRelease` may enter the final publication API.

Candidate, regtest, pilot, and production statuses must be explicit and
noninterchangeable.

> Illustrative boundary; names and exact fields are not frozen.

```rust
pub fn validate_release(
	inputs: ReleaseInputs<'_>,
	policy: &ReleasePolicy,
) -> Result<ValidatedRelease, ReleaseError>;
```

## Identity graph · `rule:release:identity-graph`

Release validates the chain:

```text
Attestation
→ architecture
→ realization
→ compiler plan
→ target/deployment
→ backend configuration
→ linked bundle
→ transaction ABI
→ vector set
→ evidence reports
→ deployment profile
→ release identity
```

A report with a mismatched identity fails even when its internal status says
`passed`.

Filenames never establish identity.

## Architecture gate · `rule:release:architecture`

Release calls architecture-owned validation for:

- architecture structure;
- supported envelope;
- final status;
- anchor-set pin;
- semantic and behavioural hashes;
- versioning gate.

Architecture JSON and TOML are:

1. parsed;
2. checked for unknown fields;
3. envelope-validated;
4. compared with typed expected value;
5. compared with exact canonical bytes.

Document appendix and masthead welds are separately checked or bound by an
artifacts report.

## Realization and compiler gates · `rule:release:semantic-scope`

Production release requires complete approved operation scope.

Require:

- realization coverage complete;
- no unresolved semantic placeholder;
- constructibility/lifecycle complete;
- model conformance passed;
- compiler relation census equal;
- no dropped relation;
- target plan complete;
- placement/layout/coverage requirements complete.

Pilot reports cannot satisfy full release.

## Target gate · `rule:release:target`

Require:

- exact target-definition identity;
- exact deployment instance;
- network/genesis equality across all reports;
- activation/configuration evidence;
- every selected target capability mapped to verified evidence.

Node implementation version is report provenance under
(`[ADR011-rule:toolchain:target-compatibility]`), not protocol identity.

## Bundle and ABI gate · `rule:release:bundle`

Require:

- final linked bundle;
- no unresolved mandatory symbol or relocation;
- complete constructor and operation scope;
- reachable relation carrier census equal to compiler scope;
- final calibrated bounds;
- bundle identity valid;
- ABI target/bundle/bounds/scope exact;
- witness and representation schemas complete;
- canonical bundle and ABI publications current.

## Calibration · `rule:release:calibration`

Release initially owns calibration orchestration to avoid a linker/transaction
dependency cycle.

For each candidate:

1. link candidate;
2. derive candidate ABI;
3. build valid worst-case transactions;
4. target-execute and measure;
5. select bounds under deterministic policy.

After selection:

1. relink final bundle;
2. regenerate final ABI;
3. regenerate every affected fixture;
4. remeasure;
5. bind exact final reports.

A bound used by several operations is valid only if every affected operation
fits.

Draft defaults are not final calibration.

## Evidence census · `rule:release:evidence`

Required report classes remain separate:

- model unit tests;
- property tests;
- model/realization conformance;
- compiler analysis;
- backend patterns;
- linked-bundle execution;
- relation coverage;
- representation safety;
- representation minimality where claimed;
- resources/calibration;
- target dependencies;
- script integration;
- independent events;
- independent query;
- independent accounting.

Required status is:

```text
passed
```

Missing, failed, incomplete, unsupported, skipped, stale, zero-hash, or
infrastructure-error reports block release.

No general waiver mechanism exists.

## Deployment profile · `rule:release:profile`

Construct the architecture-owned `DeploymentProfile`.

Populate exact:

- schema and final status;
- architecture hash;
- network/genesis;
- script limits;
- calibrated bounds;
- dependency evidence;
- artifact hashes;
- test evidence.

Call architecture deployment validation.

Compute and verify the domain-separated deployment-profile hash.

Additional compiler-era evidence not represented in profile schema remains
bound by the release manifest until a reviewed profile-schema revision adds
typed fields.

## Artifacts · `rule:release:artifacts`

Every release asset has:

- typed kind;
- canonical relative path;
- schema or media type;
- deterministic bytes;
- explicit hash recipe;
- typed source binding.

Paths are normalized, unique, relative, and free of `..`.

Source-tree and archive hashes require documented canonical recipes. Host
metadata is excluded or normalized.

## Publication · `rule:release:publication`

Publication is an explicit side effect to a caller-selected destination.

It:

1. stages into a unique directory;
2. writes the exact asset census;
3. rereads and verifies bytes/hashes;
4. validates manifest and profile;
5. atomically publishes where practical;
6. returns a publication receipt.

An existing destination is rejected by default.

The non-writing checker never repairs a release.

## Reproducibility · `rule:release:reproducibility`

Canonical release output depends only on explicit inputs.

Release date and source date are explicit.

Canonical output excludes host, path, process, timezone, credential, and
ambient-clock data.

Two clean publications from the same inputs must be byte-identical.

## Hash recipes · `rule:release:hash-recipes`

No release field is populated from an informal directory hash. Before use, each
artifact hash defines its algorithm and domain, typed or path census, canonical
relative paths, file-byte treatment, line-ending, executable-bit, symlink, and
generated-file policies, archive metadata normalization, and migration policy.
Artifact byte hashes remain distinct from semantic identities unless one
canonical serialization explicitly binds them.

## Secret boundary · `rule:release:secrets`

Release never receives production private keys.

Canonical reports and assets exclude:

- private keys;
- signing nonces;
- secret blinders;
- unpublished openings;
- RPC credentials;
- credential-bearing URLs.

## Assurance boundary · `sec:release:assurance`

Release establishes complete typed cross-binding and final profile validity.

It does not prove:

- cryptographic assumptions;
- universal compiler correctness;
- perfect target software;
- observer conceptual independence;
- network liveness;
- transaction confirmation.

Those remain named assumptions or separately scoped evidence.

## Exit gate · `gate:release:final`

A production release exits only when:

- every approved operation is implemented;
- every relation has complete bundle coverage;
- target and deployment evidence are verified;
- all bounds are final and remeasured;
- final bundle and ABI identities verify;
- generated publications are current;
- independent event, query, and accounting reports pass separately;
- architecture deployment validation passes;
- profile and release identities verify;
- publication is secret-free and byte-reproducible;
- the checker leaves the source checkout clean.

## Error vocabulary · `sec:release:errors`

See [`errors/release.md`](errors/release.md).

## Open questions · `sec:release:open`

- What is the release-manifest schema?
- When does the deployment-profile schema gain compiler-era identities?
- What canonical recipe produces the normative source-tree hash?
- What enters the compiler-configuration artifact hash?
- What is the `reference_indexer` artifact?
- What archive format and metadata normalization are canonical?
- Is release signing added, and what remains the unsigned identity?
- Does calibration remain release-owned?
