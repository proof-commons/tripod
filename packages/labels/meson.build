# Rust-source census of the labels crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build. The parent meson.build accumulates this list
# into its role group.
labels_source_files = files(
  'src/adoption.rs',
  'src/attestation.rs',
  'src/bin/census-audit.rs',
  'src/bin/check-forbidden-text.rs',
  'src/bin/check-labels.rs',
  'src/bin/check-plans.rs',
  'src/bin/generate-attestation-register.rs',
  'src/bin/generate-label-registers.rs',
  'src/census.rs',
  'src/check.rs',
  'src/diagnostic.rs',
  'src/forbidden.rs',
  'src/label.rs',
  'src/latex.rs',
  'src/lib.rs',
  'src/markdown.rs',
  'src/nearmiss.rs',
  'src/owner.rs',
  'src/participation.rs',
  'src/plans.rs',
  'src/registry.rs',
  'src/render.rs',
  'src/repository.rs',
  'src/rust_source.rs',
  'src/source.rs',
  'src/tests.rs',
)

# Same-typed files deliberately outside the crate-source census
# (ADR-014): integration-test sources are cargo test inputs, not label
# subjects. The census audit accepts the exclusion only because it is
# declared here.
labels_excluded_files = files('tests/subprocess_contract.rs')

# Crate documentation (DOC owner).
labels_doc_files = files('README.md')
