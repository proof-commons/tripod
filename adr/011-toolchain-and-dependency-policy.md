# ADR-011: Toolchain and Dependency Policy

**Status:** Decided and implemented
**Scope:** The complete Cargo workspace, current and future packages

---

## Context · `sec:toolchain:context`

The compiler-era package graph must not allow each crate to choose its own
toolchain, dependency, locking, unsafe-code, target, or reproducibility policy.

The workspace uses one deliberate policy before that graph expands.

## Rust toolchain · `rule:toolchain:msrv`

The workspace uses:

```text
Rust MSRV: 1.88
Edition:   2024
```

The MSRV is declared through workspace package metadata and inherited by every
crate.

Raising the MSRV requires a deliberate, reviewable change. It must not occur as
an incidental dependency update.

The repository does not commit `rust-toolchain.toml`.

Toolchains are supplied by the build environment.

Required CI Rust lanes are:

1. the declared MSRV;
2. current stable.

Nightly may be used for optional diagnostics or development, but no nightly
feature may become load-bearing.

## Cargo lock · `rule:toolchain:locked`

The repository does not commit `Cargo.lock`.

Before v1 the dependency surface is deliberately allowed to float within the version ranges declared in `Cargo.toml`. A floating surface is the cheaper one to keep small: every resolution takes current versions, so a dependency that has been outgrown, absorbed upstream, or left unused shows up as friction at the next build instead of being held in place by a pin. Judging a dependency change against the range the manifest declares also keeps the reviewable unit the manifest itself, which is where the decision belongs.

Resolution is therefore reproduced from the workspace manifests and the registry rather than from a committed lock, and it is not byte-stable across time. Cargo runs without `--locked`: with no lockfile present the flag has nothing to verify against and fails the invocation outright.

The pin returns as the project nears v1. A released artifact must be rebuildable from a fixed dependency surface, and at that point a committed lockfile earns the maintenance it costs — which is what the pre-v1 tree does not yet need. Reintroducing it is a deliberate step of the v1 approach, not a silent default.

Dependency changes stay deliberate commits: an addition, a removal, or a range change lands as an edit to `Cargo.toml` and is reviewed there.

## Dependency ownership · `rule:toolchain:dependencies`

First-party crates declare shared third-party dependencies in:

```toml
[workspace.dependencies]
```

Crates inherit those versions with:

```toml
dependency = { workspace = true }
```

A new dependency requires review of purpose, maintained status, source and
version, license, transitive graph, unsafe boundary, determinism impact, MSRV
impact, and advisory status.

Avoid dependencies for functionality that is small, deterministic, and clearer
with standard-library code.

Planning or reference Markdown is never a dependency configuration source.

## Licenses · `rule:toolchain:licenses`

The workspace is:

```text
AGPL-3.0-only
```

The workspace licence is copyleft, and that strengthens rather than relaxes the dependency rule below: permissive terms can be carried into a copyleft distribution unchanged, whereas an incoming copyleft term would bind the workspace to conditions it did not choose and may not be able to reconcile with the ones it did.

Dependencies must use permissive licenses compatible with repository
distribution, normally MIT, Apache-2.0, BSD, ISC, Zlib, or equivalent terms.

Copyleft dependencies are not accepted into the first-party build graph without
a separate repository policy decision.

License checking may later be mechanized. Until then it is part of dependency
review.

## Advisories · `rule:toolchain:advisories`

`cargo audit` is the advisory lane. When installed, CI runs it against the
committed lockfile.

A missing `cargo-audit` is reported loudly as a skipped lane, never as a silent
pass.

A release-blocking advisory must be resolved by an update, removed with the
affected dependency, or addressed by an explicit reviewed exception policy.
No general exception policy is established by this ADR.

## Unsafe code · `rule:toolchain:unsafe`

First-party unsafe code is denied workspace-wide.

A future package with a genuine FFI or substrate need must:

- scope an `#[allow(unsafe_code)]` to the smallest practical module;
- document the safety invariant;
- test the safe boundary;
- identify the external implementation and ownership;
- avoid weakening unrelated crates.

A dependency using unsafe internally does not make first-party code safe by
default. The dependency remains part of the reviewed trust surface.

No backend receives a blanket unsafe exception.

## Target compatibility · `rule:toolchain:target-compatibility`

The attestation realization targets the tapscript capability set deployed on
Liquid.

The project trusts the selected network's consensus and does not claim to
re-audit an Elements consensus implementation.

No particular Elements consensus-implementation source revision is protocol,
architecture, realization, target-compatibility, or deployment-release identity.

The future typed target compatibility contract may bind opcode discriminants,
stack contracts, encodings, execution domain, leaf version, transaction
interfaces, resource interfaces, and capability semantics.

Target-native integration reports record the tested node's version, build,
configuration, network, genesis, and activation state as ordinary test
provenance.

Changing the tested node revision may stale a report. It does not change
protocol identity when the typed compatibility contract and deployment instance
remain unchanged.

The reference under `plans/reference/` is explanatory only and is never a
compiler or backend input.

## Reproducibility · `rule:toolchain:reproducibility`

Identical explicit inputs must produce byte-identical generated publications,
flattened LaTeX, release documents, future realization publications, analyzed
compiler reports, emitted programs, linked bundles, transaction ABIs, canonical
vectors, and release manifests.

Canonical outputs must not depend on wall-clock time, filesystem enumeration,
hash-map order, temporary path, hostname, username, process ID, locale, or
uncontrolled environment variables.

Time or release-date material is supplied explicitly or derived from a pinned
source date such as `SOURCE_DATE_EPOCH`.

Cryptographic transaction construction may use explicit randomness. Given the
same explicit randomness and all other inputs, canonical test construction must
remain reproducible. Production randomness must not reuse deterministic test
seeds.

## Generated artifacts · `rule:toolchain:generated`

Every generated artifact has:

1. one typed source;
2. one explicit generator;
3. one non-writing checker;
4. canonical ordering;
5. deterministic bytes;
6. an owned schema;
7. unknown-field rejection where parsed externally;
8. no reverse semantic dependency.

Tests and checks do not repair tracked output.

Generated JSON, TOML, Markdown, LaTeX, label registers, vectors, and reports
remain derivative publications unless another typed external schema is
explicitly accepted.

## Clean repository gate · `rule:toolchain:clean-tree`

The canonical Rust entry point is:

```sh
scripts/ci.sh
```

It is a shim over `meson test`, which is where every lane is declared, timed,
and statused. The lanes are formatting, Clippy with `-D warnings`, per-package
debug and release tests, the census audit, generated-artifact and label checks,
planning and forbidden-text checks, the mocked Meson contract, the node-free
executor classification tests, the advisory lane, and a clean-tree check.
Arguments are forwarded to `meson test`, so a suite or a single lane is the
same command narrowed.

The advisory lane stays externally provisioned: `cargo audit` is not a
workspace dependency, and when it is absent the lane reports the harness's
SKIP status rather than a pass. A skipped lane is neither failure nor success,
and a run that skipped one is not a green run.

Document verification additionally runs the Meson document lane in a
non-mocked build directory and the reproducibility lane
(`scripts/check-document-reproducibility.sh`). Neither is part of the shim:
the first needs a real TeX toolchain, and the second builds the document twice
in disposable directories and refuses a dirty worktree, so it answers a
release question rather than a per-batch one.

A clean checkout must pass checks without modifying tracked files. An explicit
generation or publication command may write only to its requested destination.

## Rejected alternatives · `sec:toolchain:alternatives`

### Repository-pinned Rust toolchain file

Rejected because the build environment supplies MSRV and stable lanes
explicitly.

### Per-crate dependency versions

Rejected because they create hidden resolution divergence.

### Unlocked build paths

Rejected because builds must not mutate dependency resolution.

### Workspace-wide unsafe allowance

Rejected because unsafe invariants must remain narrow and reviewable.

### Elements implementation revision as protocol identity

Rejected because the project consumes a deployed capability contract and
network consensus, not one source-tree revision as protocol semantics.

### Ambient timestamps in canonical artifacts

Rejected because identical inputs must reproduce identical bytes.

## Verification · `gate:toolchain:verification`

ADR-011 is implemented when:

- every crate inherits workspace package metadata and lints;
- MSRV and stable lanes pass;
- release/build Cargo paths resolve from the workspace manifests without `--locked`;
- no lockfile is committed;
- dependencies use workspace ownership;
- first-party unsafe code remains denied except for reviewed narrow scopes;
- target plans do not reintroduce implementation-source identity;
- generated checks are non-writing;
- document and artifact reproducibility checks pass;
- the complete check leaves the repository clean.