# Generated Artifact Register · `reg:artifacts:ownership`

Generated publications are one-way derivatives. Their writer and non-writing
checker are explicit so convenient output files do not become semantic inputs.

| Artifact | Typed derivation owner | Writer | Non-writing checker | Semantic input? |
|---|---|---|---|---:|
| `architecture.json` | architecture | artifacts | `check-generated` | no |
| `architecture.toml` | architecture | artifacts | `check-generated` | no |
| `declassification.json` | model today; realization after migration | artifacts | `check-generated` | no |
| `model_labels.json` | labels | artifacts | `check-generated` / `check-labels` | no |
| `plans/labels/specification.md` | labels | labels generator | `check-labels` | no |
| `plans/labels/realization.md` | labels | labels generator | `check-labels` | no |
| flattened LaTeX | flatten-latex-main | explicit document build | reproducibility path | no |
| rendered PDF | document build | explicit document build | reproducibility gate | no |
| future compiler analysis publication | compiler | explicit generator | compiler checker | no |
| future linked bundle | linker | release/generator | linker/release checker | deployment artifact |
| future transaction ABI | transaction | release/generator | transaction/release checker | external construction input; not semantic source |
| future vectors/reports | vectors | explicit evidence runner | vectors/release checker | evidence only |
| deployment profile | release from architecture-owned type | release | release checker | release object |
| release manifest | release | release | release checker | publication index |

Phase 1 cross-checks the compact-ASH and live-transfer rows of
`declassification.json` against realization's typed pilot analysis. The full
publication remains model-owned until realization scope covers every architecture
operation.