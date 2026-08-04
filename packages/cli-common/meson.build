# Rust-source census of the cli-common crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build. The parent meson.build accumulates this list
# into its role group.
cli_common_source_files = files(
  'src/lib.rs',
  'src/publication.rs',
  'src/tests/mod.rs',
)

# Same-typed files deliberately outside the crate-source census
# (ADR-014): integration-test sources are cargo test inputs, not label
# subjects. The census audit accepts the exclusion only because it is
# declared here.
cli_common_excluded_files = files('tests/public_api.rs')

# Crate documentation (DOC owner).
cli_common_doc_files = files('README.md')
