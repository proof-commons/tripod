# Rust-source census of the tapscript crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build. The parent meson.build accumulates this list
# into its role group.
tapscript_source_files = files(
  'src/authorization.rs',
  'src/bundle.rs',
  'src/capability.rs',
  'src/error.rs',
  'src/instruction.rs',
  'src/lib.rs',
  'src/live_bundle.rs',
  'src/live_constructor.rs',
  'src/live_pattern.rs',
  'src/live_plan.rs',
  'src/live_private.rs',
  'src/live_shape.rs',
  'src/operation_assessment.rs',
  'src/pattern.rs',
  'src/policy.rs',
  'src/program.rs',
  'src/shape.rs',
  'src/stack.rs',
  'src/tests/abstract_oracle_tests.rs',
  'src/tests/authorization_tests.rs',
  'src/tests/bundle_tests.rs',
  'src/tests/byte_census_tests.rs',
  'src/tests/census_tests.rs',
  'src/tests/guide12_reproductions.rs',
  'src/tests/guide13_reproductions.rs',
  'src/tests/live_bundle_tests.rs',
  'src/tests/live_constructor_tests.rs',
  'src/tests/live_pattern_tests.rs',
  'src/tests/live_plan_tests.rs',
  'src/tests/live_private_tests.rs',
  'src/tests/live_shape_tests.rs',
  'src/tests/mapping_tests.rs',
  'src/tests/mod.rs',
  'src/tests/non_weakening_tests.rs',
  'src/tests/operation_assessment_tests.rs',
  'src/tests/parser_tests.rs',
  'src/tests/pattern_tests.rs',
  'src/tests/policy_tests.rs',
  'src/tests/public_api_tests.rs',
  'src/tests/push_census_tests.rs',
  'src/tests/schedule_tests.rs',
  'src/tests/shape_tests.rs',
  'src/tests/stack_tests.rs',
  'src/upstream.rs',
)

# Crate documentation (DOC owner).
tapscript_doc_files = files('README.md')

# Same-typed files deliberately outside the crate-source census
# (ADR-014): integration-test sources are cargo test inputs, not label
# subjects. The census audit accepts the exclusion only because it is
# declared here.
tapscript_excluded_files = files('tests/public_api.rs')
