# ADR-016: Semantic Identities, Artifact Digests, and Evidence Binding

**Status:** Proposed
**Scope:** First-party semantic identities, provenance identities, generated
and release artifacts, evidence reports, deployment profiles, and future
release authentication
**Amends:** the identity interpretation of reproducibility and generated
artifacts under (`[ADR011-rule:toolchain:reproducibility]`) and
(`[ADR011-rule:toolchain:generated]`)
**Does not establish:** authenticity, correctness, independence, or deployment
readiness merely from matching hashes

---

## Context · `sec:identity:context`

The repository uses several legitimate digests and provenance identifiers:

- Git commit and tree object IDs;
- document and instance UUIDs;
- the Layer-0 anchor-set hash;
- architecture semantic and behavioural hashes;
- generated-artifact byte comparisons;
- the deployment-profile hash;
- report and artifact hash fields reserved by the deployment profile.

Future compiler, target, bundle, ABI, vector, evidence, and release packages
could multiply these identities until fields are hashed individually and every
layer repeats every upstream digest.

That would increase maintenance without increasing assurance. A digest
establishes equality with respect to one recipe. It does not establish semantic
validity, implementation correctness, evidence independence, or authenticity.

The repository therefore needs one rule for when an object receives a digest,
who consumes it, and what decision that consumer makes.

---

## Assurance mechanisms · `rule:identity:mechanisms`

The following mechanisms are distinct:

| Mechanism | Establishes | Does not establish |
|---|---|---|
| Type | Representable shape | Cross-field validity |
| Validator | Declared object constraints and invariants | Authenticity or implementation correctness |
| Test, proof, or target execution | Evidence for a scoped claim | Identity or universal correctness |
| Semantic identity | Equality of a canonical typed projection | Validity or authenticity by itself |
| Artifact digest | Equality of exact bytes | Meaning or semantic correctness |
| Provenance identity | A named source revision, tree, or input set | Correctness of that source |
| Evidence-report identity | One typed report applies to named subjects | Honesty or implementation independence |
| Signature | A named authority approved an identity | Correctness of the signed object |
| Reproducible build | Independent builds produced equal bytes | An uncompromised toolchain |

A hash never replaces the owning type, validator, or evidence requirement.

An unkeyed digest proves no authenticity. Authenticity requires comparison with
an independently trusted expected identity or an external signature over an
accepted root identity.

---

## Digest admission · `rule:identity:admission`

A new digest or digest-bearing field is accepted only when its design names:

1. the complete typed object or exact artifact bytes identified;
2. the package owning the recipe;
3. the producer;
4. a present consumer introduced no later than the same implementation series;
5. the exact accept/reject or cache/reuse decision made by that consumer;
6. the assurance class: semantic equality, byte integrity, provenance, evidence
   binding, or release authentication;
7. canonical projection and encoding;
8. domain separator and recipe identifier;
9. exact stale conditions;
10. migration behavior when the recipe changes;
11. explicit non-claims.

A proposed digest is rejected when:

- no named consumer makes a decision from it;
- direct typed comparison is sufficient;
- the parent identity already provides the same assurance;
- the identified value has no independent storage, transport, signature, cache,
  reuse, disclosure, or versioning boundary;
- it is added only because the value is important;
- canonicalization or migration is undefined.

Fields are validated as parts of their owning object. They are not
independently hashed merely to detect changes.

---

## Identity classes · `rule:identity:classes`

### Semantic identity

A semantic identity identifies a canonical projection of a validated typed
object:

\[I_X = H(D_X \parallel V_X \parallel C(P_X(X)))\]

where:

- \(D_X\) is a domain separator;
- \(V_X\) identifies the recipe;
- \(P_X\) is the semantic projection;
- \(C\) is canonical serialization;
- \(H\) is the selected digest algorithm.

A semantic identity is appropriate when one meaning may have several
presentation encodings or is consumed independently across a package, process,
cache, or publication boundary.

### Artifact digest

An artifact digest identifies exact bytes. Its release-manifest entry binds:

- artifact role;
- canonical relative path;
- schema or media type;
- digest algorithm;
- byte digest.

An artifact digest does not become semantic identity unless one reviewed
canonical byte encoding is explicitly defined as the semantic object.

### Provenance identity

A provenance identity identifies source material, such as:

- a Git commit;
- a Git tree;
- an exact canonical publication-input set.

It remains separate from semantic and artifact identity.

### Evidence-report identity

A report identity identifies a validated typed report envelope containing at
least:

- report role;
- report schema;
- exact subject identities;
- producer or implementation identity;
- configuration where relevant;
- result status;
- canonical payload or payload digest.

A raw digest without role and subject binding is not sufficient evidence
identity.

### Release identity

A release identity identifies the canonical release manifest that aggregates:

- the deployment profile;
- required evidence references;
- distributed artifacts and byte digests;
- release policy;
- explicit source revision and release date.

If release signing is introduced, the release-manifest identity is the signing
root. Internal fields and intermediate objects are not signed separately unless
they have an independent operational authority boundary.

---

## Identity flow · `rule:identity:flow`

Semantic identity follows:

```text
authoritative typed value
    ↓
owner validation
    ↓
canonical semantic projection
    ↓
independent consumer exists?
    ├─ no  → stop; do not hash
    └─ yes → domain-separated semantic identity
                 ↓
             immediate consumer binding
```

Publication follows separately:

```text
validated typed value
    ↓
canonical renderer
    ↓
artifact bytes
    ├─ exact freshness comparison
    └─ byte digest only when independently distributed or release-bound
```

Evidence follows separately:

```text
test or analysis execution
    ↓
typed report payload
    ↓
validated report envelope
    ↓
report identity
    ↓
deployment profile or release manifest
```

Exact expected-byte comparison remains sufficient for committed generated
publications. Such a comparison does not require an additional semantic or
per-file hash.

---

## Immediate dependency edges · `rule:identity:immediate-edges`

An independently consumed parent binds only its immediate identity
dependencies.

Conceptually:

```text
ArchitectureSemanticId
    → RealizationId
    → CompilerPlanId
    → TargetPlanId
    → LinkedBundleId
    → TransactionAbiId
    → DeploymentProfileId
    → ReleaseManifestId
```

A parent does not repeat every transitive upstream identity. Human-readable
manifests may display the complete chain, but authoritative validation follows
immediate typed edges.

A child receives an independent identity only when it has an independent
lifecycle. Otherwise the parent includes the canonical typed child value
directly.

No Petgraph index, source order, path, line number, solver variable number,
matrix position, traversal order, thread schedule, temporary path, or
floating-point working value enters semantic identity.

---

## Producer and consumer validation · `rule:identity:verification`

The owning producer:

1. validates the complete typed object;
2. derives its canonical projection;
3. computes its identity;
4. publishes the object and recipe identifier together where external
   consumption exists.

An immediate consumer:

1. parses external bytes into a typed value when necessary;
2. rejects unknown fields and unsupported schemas;
3. runs the owner's validator;
4. recomputes the identity;
5. compares the required immediate dependency identity;
6. consumes the typed value.

The release validator traverses the typed identity graph, calls package-owned
validators, verifies immediate edges, checks required evidence roles, and
verifies artifact bytes. It does not reimplement every package validator.

---

## Evidence binding and independence · `rule:identity:evidence`

A report digest binds a report to its role and subjects. It does not prove that
the implementation producing the report is independent.

Independence remains a reviewed provenance claim recording implementation
identity, shared code and dependencies, operator, and execution environment
where relevant.

Digest inequality is not evidence of independence. Equal report hashes must not
be rejected merely to manufacture an appearance of independence; different
report roles are distinguished by typed report envelopes and domain separation.

Ordinary local CI logs require no persistent report identity unless another
package consumes them as release evidence.

---

## Current identities · `rule:identity:current`

Current identities have these scopes:

| Identity | Purpose | Policy |
|---|---|---|
| Git commit/tree IDs | Source provenance | Retain; never protocol identity |
| Document UUID | Exact paper-input provenance in XMP | Retain; publication-only |
| Instance UUID | Paper-subtree Git-tree provenance in XMP | Retain; publication-only |
| Layer-0 anchor-set hash | Exact imported Layer-0 dependency set | Retain |
| Architecture semantic hash | Canonical complete architecture meaning | Retain |
| Architecture behavioural hash | Realization-major versioning gate only | Retain; do not propagate as a general runtime identity |
| Generated-file exact comparisons | Publication freshness | Retain; add no redundant hash |
| Deployment-profile hash | Future aggregate deployment-profile identity | Retain as pre-release infrastructure; it gives no release assurance until a real consumer validates it |
| Raw artifact/report hash fields in profile schema 2 | Provisional references | Must receive owned recipes and typed report/artifact references before production release |

Document provenance identities must not enter realization, compiler, target,
bundle, ABI, or protocol identities.

Before production release, deployment calibration must bind the exact final
bundle and transaction ABI. Evidence fields must bind typed report roles and
subjects rather than rely on bare digest arrays.

---

## Recipe migration · `rule:identity:migration`

A published identity recipe is never silently redefined.

Changing its:

- projection;
- canonical encoding;
- domain separator;
- digest algorithm;
- included semantic fields;
- exclusion rules

creates a new recipe identifier.

Migration records:

- old and new recipes;
- reason;
- whether meaning changed or only measurement changed;
- old and new identities where applicable;
- consumer transition policy.

A recipe migration does not itself imply a semantic version change. The owning
semantic versioning rule decides that question.

---

## Consequences · `sec:identity:consequences`

This decision yields:

- one aggregate identity per independently meaningful object;
- one byte digest per independently distributed artifact;
- typed evidence references instead of ambiguous raw hashes;
- immediate rather than all-to-all dependency binding;
- one release root for future authentication;
- no field-level hash or checker proliferation;
- explicit assurance and non-claim boundaries.

Types and validators remain the primary correctness mechanism. Hashes remain
comparison and binding mechanisms.

---

## Rejected alternatives · `sec:identity:alternatives`

### Hash every field

Rejected because fields do not have independent lifecycles, cross-field
validity still requires object validation, and field hashes create no
additional correctness evidence.

### One undifferentiated global repository hash

Rejected because semantic identity, artifact bytes, provenance, evidence, and
release authentication have different stale conditions and consumers.

### All-to-all hash binding

Rejected because it duplicates transitive dependencies and creates an
\(O(n^2)\) consistency mesh. Immediate typed edges provide the same transitive
binding with clearer ownership.

### Bare digest arrays

Rejected for production evidence because the digest alone does not identify
role, schema, subject, producer, or assurance class.

### Hash inequality as independence evidence

Rejected because different bytes do not prove independent implementation or
judgment.

### Hashes as validation

Rejected because self-consistent invalid objects can be rehashed. Validation
precedes hashing.

### Per-component signatures

Rejected unless a component has an independent authority boundary. Future
release authentication signs the release root.

---

## Verification · `gate:identity:verification`

This decision is implemented when:

- every existing digest is classified by object, owner, producer, consumer,
  decision, assurance, stale condition, migration, and non-claims;
- no new digest enters without satisfying (`rule:identity:admission`);
- semantic, artifact, provenance, evidence, and release identities use distinct
  typed roles;
- generated-publication freshness does not acquire redundant hashes;
- future identity graphs bind immediate dependencies only;
- Petgraph and other local handles remain absent from semantic identity;
- evidence reports bind typed roles and exact subjects;
- deployment calibration binds the final bundle and ABI before production;
- release validation delegates to package-owned validators;
- any future signature authenticates the release-manifest identity;
- full repository checks remain green and clean.
