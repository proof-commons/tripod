# Rust-source census of the artifacts crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build. The parent meson.build accumulates this list
# into its role group.
artifacts_source_files = files(
  'src/bin/check-generated.rs',
  'src/bin/generate-all.rs',
  'src/lib.rs',
  'src/tests/mod.rs',
  'src/tests/weld_tests.rs',
  'src/weld.rs',
)

# Same-typed files deliberately outside the crate-source census
# (ADR-014): integration-test sources are cargo test inputs, not label
# subjects. The census audit accepts the exclusion only because it is
# declared here.
artifacts_excluded_files = files('tests/subprocess_contract.rs')
