# Rust-source census of the target-elements crate (ADR-014): explicit
# and hand-managed — a new source file must be listed here, or the
# census audit fails the build. The parent meson.build accumulates this
# list into its role group.
target_elements_source_files = files(
  'src/authorization.rs',
  'src/capability.rs',
  'src/confidential.rs',
  'src/definition.rs',
  'src/encoding.rs',
  'src/error.rs',
  'src/evidence.rs',
  'src/evidence_registry.rs',
  'src/lib.rs',
  'src/opcode.rs',
  'src/resource.rs',
  'src/tests/capability_tests.rs',
  'src/tests/definition_tests.rs',
  'src/tests/encoding_tests.rs',
  'src/tests/mod.rs',
  'src/tests/mutation_tests.rs',
  'src/tests/opcode_tests.rs',
  'src/tests/permutation_tests.rs',
  'src/tests/registry_mutation_tests.rs',
)

# Crate documentation (DOC owner).
target_elements_doc_files = files('README.md')

# Same-typed files deliberately outside the crate-source census
# (ADR-014): integration-test sources are cargo test inputs, not label
# subjects. The census audit accepts the exclusion only because it is
# declared here.
target_elements_excluded_files = files('tests/public_api.rs')
