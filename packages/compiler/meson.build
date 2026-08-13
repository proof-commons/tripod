# Rust-source census of the compiler crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build. The parent meson.build accumulates this list
# into its role group.
compiler_source_files = files(
  'src/analyzed.rs',
  'src/analyzed_operation.rs',
  'src/analyzed_validate.rs',
  'src/capability.rs',
  'src/carrier.rs',
  'src/case.rs',
  'src/constructibility.rs',
  'src/coverage.rs',
  'src/coverage_graph.rs',
  'src/disclosure.rs',
  'src/error.rs',
  'src/expression.rs',
  'src/fold.rs',
  'src/foundation.rs',
  'src/input.rs',
  'src/layout.rs',
  'src/lib.rs',
  'src/lifecycle.rs',
  'src/placement.rs',
  'src/proof.rs',
  'src/relation.rs',
  'src/requirement.rs',
  'src/search_counter.rs',
  'src/source.rs',
  'src/sponsor_region.rs',
  'src/tests/analyzed_corruption_tests.rs',
  'src/tests/analyzed_operation_tests.rs',
  'src/tests/analyzed_program_oracle_tests.rs',
  'src/tests/analyzed_program_tests.rs',
  'src/tests/carrier_tests.rs',
  'src/tests/case_tests.rs',
  'src/tests/constructibility_tests.rs',
  'src/tests/coverage_graph_tests.rs',
  'src/tests/coverage_oracle_tests.rs',
  'src/tests/coverage_tests.rs',
  'src/tests/disclosure_tests.rs',
  'src/tests/expression_tests.rs',
  'src/tests/fold_tests.rs',
  'src/tests/layout_tests.rs',
  'src/tests/lifecycle_tests.rs',
  'src/tests/mod.rs',
  'src/tests/oracle_tests.rs',
  'src/tests/pilot_coverage_tests.rs',
  'src/tests/pilot_placement_tests.rs',
  'src/tests/pilot_program_tests.rs',
  'src/tests/placement_oracle_tests.rs',
  'src/tests/placement_tests.rs',
  'src/tests/proof_tests.rs',
  'src/tests/relation_tests.rs',
  'src/tests/requirement_tests.rs',
  'src/tests/source_tests.rs',
)

# Crate documentation (DOC owner).
compiler_doc_files = files('README.md')

# Integration-test sources are cargo test inputs rather than label
# graph subjects. The explicit exclusion keeps the tracked-file census
# welded to that classification.
compiler_excluded_files = files('tests/public_api.rs')
