# Rust-source census of the realization crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build. The parent meson.build accumulates this list
# into its role group.
realization_source_files = files(
  'src/binding.rs',
  'src/domain.rs',
  'src/error.rs',
  'src/identity.rs',
  'src/lib.rs',
  'src/scope.rs',
  'src/tests/mod.rs',
)

# Crate documentation (DOC owner).
realization_doc_files = files('README.md')

# Integration-test sources are cargo test inputs rather than label
# graph subjects. The explicit exclusion keeps the tracked-file census
# welded to that classification.
realization_excluded_files = files('tests/public_api.rs')