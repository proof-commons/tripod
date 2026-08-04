# Rust-source census of the compiler crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build. The parent meson.build accumulates this list
# into its role group.
compiler_source_files = files(
  'src/error.rs',
  'src/input.rs',
  'src/lib.rs',
)

# Crate documentation (DOC owner).
compiler_doc_files = files('README.md')

# Integration-test sources are cargo test inputs rather than label
# graph subjects. The explicit exclusion keeps the tracked-file census
# welded to that classification.
compiler_excluded_files = files('tests/public_api.rs')
