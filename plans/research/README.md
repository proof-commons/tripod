# Research Questions

This directory contains unresolved implementation questions requiring source
review, prototypes, measurements, or formal analysis.

Research notes do not define production policy.

## Index · `tbl:research:index`

| Question | Status | Blocks |
|---|---|---|
| [compiler-algorithms.md](compiler-algorithms.md) | Design and prototype required | Compiler graph, identity, proof, placement, and coverage algorithms. |
| [linker-algorithms.md](linker-algorithms.md) | Design and prototype required | Linker graph, SCC, relocation, taptree, and resource algorithms. |
| [numerical-linear-algebra.md](numerical-linear-algebra.md) | Dependency review and prototype required | Exact/numerical solver and certification policy. |
| [optimization-solvers.md](optimization-solvers.md) | Open; exact prototype required | Proof, placement, calibration, and external solver policy. |
| [state-constructor.md](state-constructor.md) | Prototype required | STATE-spending backend operations. |
| [wide-arithmetic.md](wide-arithmetic.md) | Prototype and measurement required | Redemption, settlement, and cycle arithmetic. |
| [public-declassification.md](public-declassification.md) | Open; prototype required | Confidential-to-public lifecycle paths. |
| [settlement-layout.md](settlement-layout.md) | Open; prototype required | Settlement ABI and calibrated batch size. |
| [attestation-floor-bounds.md](attestation-floor-bounds.md) | Analysis accepted; regressions landed; correction pending | Attestation containment and seigniorage claims, and the SP5 capacity export. |
| [notation-semantic-census.md](notation-semantic-census.md) | Resolved — over-scoped; the existing label graph is sufficient | Nothing further; surviving work is attestation label minting and editorial repair. |

## Note form · `rule:research:form`

Each note contains one decisive question, fixed constraints, candidate matrix,
threat model, prototype, required vectors, measurements, acceptance criteria,
rejection criteria, result, and decision handoff. Pending result sections
remain present and say `Pending`.

## Status rule · `rule:research:status`

A successful prototype does not automatically become a production interface.
Resolution requires reproducible evidence, an accepted decision or package
contract update, permanent tests, identity/schema review, and release-evidence
handoff.

## Labels · `rule:research:labels`

Primary questions use `q:<area>:<name>` and candidate labels may use
`candidate:<area>:<name>`. Research labels are plan-local,
non-normative, and non-identity-bearing; uniqueness and citation checks
still apply.

## Machine use · `rem:research:machine-use`

Research Markdown and prototype reports are not semantic input. Accepted
implementation is represented separately in typed source and tests.