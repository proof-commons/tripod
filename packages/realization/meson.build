# Rust-source census of the realization crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build. The parent meson.build accumulates this list
# into its role group.
realization_source_files = files(
  'src/binding.rs',
  'src/constructibility.rs',
  'src/declarations/compact_ash.rs',
  'src/declarations/mod.rs',
  'src/declarations/transfer_live.rs',
  'src/declassification.rs',
  'src/derive.rs',
  'src/domain.rs',
  'src/error.rs',
  'src/evaluate.rs',
  'src/expression.rs',
  'src/identity.rs',
  'src/lifecycle.rs',
  'src/lib.rs',
  'src/observation.rs',
  'src/operation.rs',
  'src/relation.rs',
  'src/scope.rs',
  'src/tests/compact_ash_tests.rs',
  'src/tests/constructibility_tests.rs',
  'src/tests/disclosure_tests.rs',
  'src/tests/derivation_tests.rs',
  'src/tests/expression_tests.rs',
  'src/tests/lifecycle_tests.rs',
  'src/tests/live_transfer_tests.rs',
  'src/tests/mod.rs',
  'src/tests/property_graph_tests.rs',
  'src/tests/type_tests.rs',
  'src/validate.rs',
  'src/value.rs',
)

# Crate documentation (DOC owner).
realization_doc_files = files('README.md')

# Integration-test sources are cargo test inputs rather than label
# graph subjects. The explicit exclusion keeps the tracked-file census
# welded to that classification.
realization_excluded_files = files('tests/public_api.rs')