# Rust-source census of the vectors crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build. The parent meson.build accumulates this list
# into its role group.
vectors_source_files = files(
  'src/bundle.rs',
  'src/comparison.rs',
  'src/divergence.rs',
  'src/error.rs',
  'src/fixture.rs',
  'src/lib.rs',
  'src/materialize.rs',
  'src/matrix.rs',
  'src/mutation.rs',
  'src/operation.rs',
  'src/plan.rs',
  'src/projection.rs',
  'src/resource_study.rs',
  'src/subject.rs',
  'src/violation.rs',
)

# Crate documentation (DOC owner).
vectors_doc_files = files('README.md')

# Same-typed files deliberately outside the crate-source census
# (ADR-014): integration-test sources are cargo test inputs, not label
# subjects. The census audit accepts the exclusion only because it is
# declared here.
vectors_excluded_files = files(
  'tests/compact_ash_operation.rs',
  'tests/guide13_preflight.rs',
)
