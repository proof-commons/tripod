# Rust-source census of the target-elements-conformance crate
# (ADR-014): explicit and hand-managed — a new source file must be
# listed here, or the census audit fails the build. The parent
# meson.build accumulates this list into its role group.
target_elements_conformance_source_files = files(
  'src/bin/check-target-elements-native.rs',
  'src/bin/mock-native-executor.rs',
  'src/census/author.rs',
  'src/census/compound.rs',
  'src/census/constructor.rs',
  'src/census/context.rs',
  'src/census/crypto.rs',
  'src/census/encoding.rs',
  'src/census/hashing.rs',
  'src/census/introspection.rs',
  'src/census/material.rs',
  'src/census/mod.rs',
  'src/census/numeric.rs',
  'src/census/timelock.rs',
  'src/claim.rs',
  'src/constructor/canonical.rs',
  'src/constructor/curve.rs',
  'src/constructor/internal_key.rs',
  'src/constructor/metadata.rs',
  'src/constructor/metadata_leaf.rs',
  'src/constructor/mod.rs',
  'src/constructor/tagged.rs',
  'src/constructor/totality.rs',
  'src/constructor/tree.rs',
  'src/error.rs',
  'src/executor.rs',
  'src/fixture.rs',
  'src/lib.rs',
  'src/protocol.rs',
  'src/prototype.rs',
  'src/prototype_program.rs',
  'src/report.rs',
  'src/tests/canonical_tests.rs',
  'src/tests/census_tests.rs',
  'src/tests/constructor_matrix_tests.rs',
  'src/tests/constructor_tests.rs',
  'src/tests/fixture_tests.rs',
  'src/tests/mod.rs',
  'src/tests/protocol_tests.rs',
  'src/tests/prototype_program_tests.rs',
  'src/tests/prototype_tests.rs',
  'src/tests/support.rs',
  'src/tests/validate_tests.rs',
  'src/tests/vocabulary_tests.rs',
  'src/validate.rs',
  'src/vocabulary.rs',
)

# Same-typed files deliberately outside the crate-source census
# (ADR-014): integration-test sources are cargo test inputs, not label
# subjects. The census audit accepts the exclusion only because it is
# declared here.
target_elements_conformance_excluded_files = files(
  'tests/executor_protocol.rs',
  'tests/executor_supervision.rs',
  'tests/subprocess_contract.rs',
)

# Crate documentation (DOC owner).
target_elements_conformance_doc_files = files('README.md')
