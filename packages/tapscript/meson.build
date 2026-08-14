# Rust-source census of the tapscript crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build. The parent meson.build accumulates this list
# into its role group.
tapscript_source_files = files(
  'src/capability.rs',
  'src/error.rs',
  'src/lib.rs',
  'src/tests/census_tests.rs',
  'src/tests/mapping_tests.rs',
  'src/tests/mod.rs',
  'src/tests/non_weakening_tests.rs',
  'src/tests/public_api_tests.rs',
)

# Crate documentation (DOC owner).
tapscript_doc_files = files('README.md')

# Same-typed files deliberately outside the crate-source census
# (ADR-014): integration-test sources are cargo test inputs, not label
# subjects. The census audit accepts the exclusion only because it is
# declared here.
tapscript_excluded_files = files('tests/public_api.rs')
