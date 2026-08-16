# Technical References

This directory contains external technical reference material used for human
review.

References are not project authority, compiler input, target identity, or
deployment evidence.

## Index · `tbl:reference:index`

| Reference | Status |
|---|---|
| [draft-gap-census.md](draft-gap-census.md) | Active review reference |
| [elements-tapscript.md](elements-tapscript.md) | Active review reference |

## Reference rule · `rule:reference:authority`

A reference may summarize external source locations, capability families, known
semantics, review provenance, known gaps, and tests still required.

Machine-consumed target facts live in typed packages. The Elements compatibility
contract is owned by `tripod-target-elements`.

Node implementation revisions used by integration tests are test provenance,
not protocol identity, under (`[ADR011-rule:toolchain:target-compatibility]`).

## Provenance · `rule:reference:provenance`

Copied external material requires upstream repository, reviewed source location,
license compatibility, attribution, and a modification note where applicable.
Prefer concise summaries and links over copied source text.

## Machine use · `rem:reference:machine-use`

No compiler, backend, linker, transaction builder, vector generator, or release
validator parses this directory.