# Rust-source census of the execwrap crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build. The parent meson.build accumulates this list
# into its role group.
execwrap_source_files = files(
  'src/bin/execwrap.rs',
  'src/bin/execwrap-mock-tex.rs',
  'src/lib.rs',
  'src/tests/mod.rs',
  'src/writer.rs',
)

# Same-typed files deliberately outside the crate-source census
# (ADR-014): integration-test sources are cargo test inputs, not label
# subjects. The census audit accepts the exclusion only because it is
# declared here.
execwrap_excluded_files = files('tests/subprocess_contract.rs')

# Crate documentation (DOC owner).
execwrap_doc_files = files('README.md')
