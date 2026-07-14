# ADR-011: Toolchain and Dependency Policy

**Status:** Decided
**Scope:** the whole Cargo workspace, present and future crates

Settled once, before the realization/compiler package graph grows,
so no new crate invents its own policy.

## Toolchain

- **MSRV is `1.88`**, declared as `rust-version` in the workspace
  `[workspace.package]` and inherited by every crate. Raising it is a
  deliberate, commit-messaged decision, never a side effect.
- **No `rust-toolchain.toml`.** The toolchain is supplied by the
  build environment (developers may use stable or nightly). CI runs
  two lanes: the declared MSRV and current stable. A nightly-only
  feature must not become load-bearing.
- **Edition 2024** workspace-wide.

## Cargo.lock

- The repository does not commit `Cargo.lock`.
- Every CI, release, and document build runs Cargo with `--locked`
  (enforced in `scripts/ci.sh`, the Meson test lanes, and
  `scripts/cargo-bin-sync.sh`). A build must never silently update
  dependency resolution.
- Updates happen through deliberate `cargo update` (or targeted
  `cargo update -p <crate>`) commits whose message says why. Adding a
  dependency updates the lock in the same commit that adds it.

## Dependencies

- **Additions are deliberate.** New dependencies go through
  `[workspace.dependencies]` with a single workspace-wide version;
  crates inherit with `{ workspace = true }`.
- **Advisories:** `cargo audit` is the advisory lane. It runs in CI
  wherever the tool is available (`scripts/ci.sh` treats a missing
  binary as a skipped, loudly-reported lane, not a silent pass). An
  advisory against a locked dependency blocks release builds until
  resolved or explicitly waived in a commit message.
- **Licenses:** the workspace is `MIT OR Apache-2.0`. Dependencies
  must be permissively licensed (MIT/Apache-2.0/BSD/ISC/Zlib or
  equivalent); copyleft is not accepted into the build graph. A
  `cargo deny` configuration may mechanize this later; until then the
  check is part of dependency-addition review.

## Unsafe code

- `unsafe` is **denied workspace-wide** (`[workspace.lints.rust]
  unsafe-code = "deny"`). All current crates are unsafe-free. A future
  crate with a genuine need (FFI to Elements, for instance) must
  scope an `#[allow(unsafe_code)]` to the smallest possible module
  and document the invariant it upholds.

## Target substrate compatibility

- the attestation realization targets the tapscript capability set deployed on
  Liquid mainnet. The required opcodes are established network
  capabilities, not semantics owned or audited by this repository.
- **No particular Elements consensus-implementation source revision is
  part of the protocol identity or a deployment-release pin.** The
  future `target-elements` crate is a typed backend compatibility
  contract (opcode discriminants, stack contracts, encodings, leaf
  version and execution domain, transaction and resource interfaces),
  not a consensus source audit.
- Backend integration tests run against a supported Elements/Liquid
  node and record its version as ordinary test provenance. Those tests
  establish compatibility of emitted artifacts; they do not claim to
  audit consensus conformance.
- First-party Rust library dependencies remain pinned through
  `Cargo.lock` (above). The survey in
  `plans/reference/elements-tapscript.md` is explanatory reference
  material, never a pin or compiler input.

## Reproducibility expectations

- Generated artifacts, flattened LaTeX, and (eventually) emitted
  script bundles must be byte-identical for identical inputs: no
  wall-clock timestamps, no environment-dependent output, canonical
  ordering everywhere.
- The canonical local verification entry point is `scripts/ci.sh`; a
  clean checkout must pass it without modifying tracked files
  (`git diff --exit-code` is part of the script).
