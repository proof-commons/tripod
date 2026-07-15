# `tripod-labels`

`labels` is the repository's documentation-conformance package. It harvests
typed labels, validates imported citations and architecture/document welds, and
renders deterministic label publications. It is documentation tooling only; no
semantic package consumes its registries or planning labels.

## Inputs

- `architecture::ARCHITECTURE`;
- Attestation LaTeX and realization Markdown;
- ADR and planning Markdown;
- model Rust source; and
- committed label registers and model-label publication.

## Outputs

The library returns owner-aware registries and diagnostics. It also renders the
specification register, realization register, and the derived model-label JSON.

`check-labels` is non-writing and emits one JSON report on stdout. It validates
source labels, imported citations, the architecture-to-document weld, the
pinned attestation anchor set, and generated-publication freshness.

`generate-label-registers` writes only:

- `plans/labels/specification.md`; and
- `plans/labels/realization.md`.

Plan-local labels remain non-normative and are not linted. The `artifacts`
package remains responsible for writing `packages/model/generated/model_labels.json`.