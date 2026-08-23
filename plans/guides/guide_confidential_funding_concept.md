# Draft: Confidential Test Materialization and Funding Protocol Concept

> **Status:** Concept draft; not an execution guide; unnumbered
> **Phase:** UNNUMBERED — sequencing relative to the already-queued next concept and the owner-sighash work is decided at charter time by the project owner
> **Entry:** the reproduced `NoConfidentialPredecessorCanBeFunded` candidate-pipeline blocker and an accepted charter decision on test-material custody
> **Primary semantic operation:** materialize and fund confidential candidate test receipts
> **Affected packages:** `target-elements-conformance`, `transaction`, `vectors`; the native Elements executor where it implements the first-party funding wire protocol
> **Evidence support:** the first-party target executor and validated-report boundary, the Guide-13 evidence registers, and Elements consensus behavior verified at reviewed tip `b7fc5d080a`
> **May affect after acceptance:** funding schema, executor capabilities, candidate transaction construction, output-witness serialization, test-opening custody, canonical report exclusions, evidence plans, package contracts, Meson graph, and dependency graph
> **Supersedes as concept direction:** scalar-only funding and per-output confidential commitment materialization for Guide-13 private candidate evidence
> **Does not implement:** the owner sighash, production wallets, production opening custody, production blinding, production signing or blinding coordination, privacy guarantees, final calibration, production deployment, or release
> **Required result:** one reviewed candidate-only protocol that creates, mines, reads back, spends, and evidences explicit-protocol-asset/confidential-value receipts with valid rangeproofs, empty surjection proofs, transaction-wide balanced value blinders, proof-bearing output witnesses finalized before owner signing, and honest evidence boundaries
> **Authority:** Attestation, the typed architecture, implemented ADRs, and accepted decisions take precedence over this concept; the project owner resolves every charter decision recorded below
> **Trust boundary:** repository source and selected executors are executable; untrusted contributions run only in an externally established, credential-free environment under ADR-015, and any selected secret-bearing interface must pass ADR-015's separate design gate before implementation

---

## Mission · `sec:guide-ctf:mission`

This concept charters the work needed to make the Guide-13 private receipt form creatable, spendable, and evidencable end to end in the candidate pipeline while preserving honest separation between funding evidence, transfer acceptance, disclosure minimality, and production readiness.
