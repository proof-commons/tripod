# ADR-026: Tracing as a Core Workspace Dependency

**Status:** Decided and implemented
**Scope:** Every current and future workspace package, and instrumentation in new code

---

## Context · `sec:tracing:context`

The repository reports where wall time goes. Separate counter types, printed progress lines and local timing code give those observations incompatible shapes, and require changes to instrumented code when collection changes. ADR-010 already establishes JSON tracing diagnostics on stderr (`[ADR010-rule:output:streams]`). This record fixes the instrumentation vocabulary without choosing a collector.

## Decision · `dec:tracing:core`

Every workspace package declares `tracing = { workspace = true }` in its ordinary, unconditional `dependencies` table. The dependency is not optional and inherits the version in the root workspace manifest.

New code records counters, timings and progress through `tracing` spans and events: span durations represent timings; events carry structured fields for counts and outcomes. New bespoke counter types, printed progress lines and ad hoc elapsed-time instrumentation are not the instrumentation interface. This does not replace domain values or command result data with diagnostics.

Installing or configuring a subscriber, and choosing what collects or reports spans and events, are separate decisions outside this change. Existing code is not ported here; it migrates only when a later change touches it for its own reasons.

## Consequences · `rem:tracing:consequences`

One structured vocabulary for counts and durations lets a later subscriber collect observations without changing the instrumented code. With no subscriber installed, tracing has near-zero runtime cost; instrumentation must still avoid eagerly computing expensive field values. Span timing directly serves the repository's practice of reporting where wall time goes.

Having the dependency in every package means instrumenting new code never waits on a manifest change. Retaining it before a package's first use is a deliberate exception to removing unused dependencies, justified by this shared instrumentation contract. It follows workspace ownership under (`[ADR011-rule:toolchain:dependencies]`). Architecture and target-elements retain an explicit `use tracing as _;` anchor in their crate roots until their first instrumentation replaces it. These narrowly acknowledge the intended dependency without suppressing Cargo's unused-dependency lint for other dependencies. The gate toolchain has no per-dependency intent setting for that lint; its transitive-dependency exemption already covers the other currently uninstrumented packages. The dependency adds build graph surface even where it has no runtime consumer.

## Rejected alternatives · `rem:tracing:alternatives`

Adding the dependency only on first use leaves policy adoption dependent on per-package manifest changes. Bespoke counters and timers couple collection to callers and fragment the structured vocabulary. Choosing a subscriber here would couple instrumentation policy to a separate reporting decision. A wholesale migration would expand this change beyond its policy purpose.

## Verification · `gate:tracing:verification`

The `workspace-tracing` lint lane reads the workspace member manifests and requires an unconditional, non-optional inherited tracing dependency in each. Its control fixture passes; removing only that dependency from one member fails with that member's manifest named. Optional, development-only and target-only declarations cannot satisfy the rule. New packages enter the check through the workspace member list, not a second package roster.

The ADR census includes this record, its index and contributor policy link resolve, and the workspace Clippy and documentation gates pass. The Clippy battery must emit no unused-tracing dependency warning; warnings about other dependencies remain findings. The manifest check enforces dependency presence; review enforces the new-code policy.
