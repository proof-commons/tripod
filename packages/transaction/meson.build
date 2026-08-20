# Rust-source census of the transaction crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build. The parent meson.build accumulates this list
# into its role group.
transaction_source_files = files(
  'src/abi.rs',
  'src/bytes.rs',
  'src/construct.rs',
  'src/error.rs',
  'src/lib.rs',
  'src/request.rs',
  'src/sponsor.rs',
  'src/synthetic.rs',
  'src/taproot.rs',
  'src/tests/abi_tests.rs',
  'src/tests/construction_tests.rs',
  'src/tests/encoding_tests.rs',
  'src/tests/mod.rs',
  'src/tests/taproot_tests.rs',
  'src/view.rs',
)

# Crate documentation (DOC owner).
transaction_doc_files = files('README.md')

# Same-typed files deliberately outside the crate-source census
# (ADR-014). The crate declares none yet; the empty list keeps the
# parent's role composition uniform across crates.
transaction_excluded_files = files()
