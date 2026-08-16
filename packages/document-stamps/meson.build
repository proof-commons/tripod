# Rust-source census of the document-stamps crate (ADR-014): explicit
# and hand-managed — a new source file must be listed here, or the
# census audit fails the build. The parent meson.build accumulates this
# list into the crate-source role group.
document_stamps_source_files = files(
  'src/bin/attestation-stamps.rs',
  'src/lib.rs',
  'src/tests/mod.rs',
)

# Same-typed files deliberately outside the crate-source census
# (ADR-014): integration-test sources are cargo test inputs, not label
# subjects. The census audit accepts the exclusion only because it is
# declared here.
document_stamps_excluded_files = files('tests/subprocess_contract.rs')

# Crate documentation (DOC owner).
document_stamps_doc_files = files('README.md')
