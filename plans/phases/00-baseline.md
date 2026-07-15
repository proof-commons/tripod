# Phase 0 — Planning Reset and Baseline · `phase:roadmap:baseline`

> **Status:** Complete
> **Completed:** 2026-07-15
> **Package work:** none
> **Primary policies:** ADR-010, ADR-011

## Goal · `sec:phase0:goal`

Establish a trustworthy compiler-era starting point before adding semantic,
compiler, or backend packages.

The phase repaired current implementation boundaries, made repository checks
reproducible, rewrote the planning hierarchy, and recorded one immutable
baseline.

## Deliverables · `sec:phase0:deliverables`

### Repository policy

- command-line output contract implemented;
- toolchain, dependency, locking, unsafe-code, target-compatibility, and
  reproducibility policy implemented;
- Cargo lock committed and used with `--locked`;
- first-party unsafe code denied.

### CLI and process integrity

- JSON help/version/usage paths;
- panic payload hidden by default;
- unredirected child streams preserved;
- child argv not logged;
- wrapper data loss fails the wrapper;
- ambiguous redirection rejected before side effects.

### Architecture and publication integrity

- supported-envelope and release-envelope checks separated from hash
  self-consistency;
- architecture document metadata validated;
- duplicate set-like declarations rejected;
- generated artifact writer/checker split preserved;
- collision-safe staged writes;
- architecture JSON/TOML and realization appendix welded to typed source.

### Model and evidence integrity

- genesis clearing required by the indexer event model;
- event/query/accounting differential APIs remain separate;
- documentation no longer overclaims independent deployment implementations;
- label delimiter failures reject rather than vanish.

### Reproducibility

- PDF date material derives from an explicit source epoch;
- two clean document builds produce identical PDF bytes;
- flattened LaTeX and generated artifacts remain deterministic;
- checks leave tracked files unchanged.

## Baseline identities · `sec:phase0:identities`

The baseline record is the set of identities recorded below, together with the canonical architecture publications they were measured over.

Planning documents do not duplicate all current hash values.

The baseline precedes later behavioural-hash algorithm migrations. Current
identities are read from typed/generated source rather than this historical
phase card.

## Evidence · `sec:phase0:evidence`

The completed gate included:

- Rust MSRV lane;
- current stable Rust lane;
- formatting;
- Clippy with `-D warnings`;
- debug and release tests;
- generated-artifact check;
- planning/documentation check;
- Meson document build;
- document reproducibility check;
- clean-tree check.

A missing optional advisory tool was reported as a skip rather than a pass,
under ADR-011.

## Reopened findings · `sec:phase0:reopened`

Static review after the baseline identified additional localized findings.

They are tracked in the active backlog and must close before Phase 1 exits:

- `Ratio` validity and zero-divisor panic;
- checkpoint/query context validation;
- history event-type ingestion boundary;
- dangling-symlink redirection aliasing;
- credential-URL redaction gaps;
- genesis/transition transaction-ID collision;
- runtime bound minima.

Reopening these findings does not rewrite the immutable baseline record. It
records that the baseline’s assurance claim was later refined.

## Exit gate · `gate:phase0:exit`

Phase 0 remains complete as a historical gate because:

- its original required lanes passed;
- its baseline tag is immutable;
- later findings are separately tracked;
- no later finding is silently edited into historical evidence.

Phase 1 cannot exit until every release-relevant reopened finding is resolved.
