# Rust-source census of the vectors crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build. The parent meson.build accumulates this list
# into its role group.
vectors_source_files = files(
  'src/bundle.rs',
  'src/error.rs',
  'src/fixture.rs',
  'src/lib.rs',
  'src/materialize.rs',
  'src/matrix.rs',
  'src/plan.rs',
  'src/projection.rs',
  'src/subject.rs',
)

# Crate documentation (DOC owner).
vectors_doc_files = files('README.md')

# Same-typed files deliberately outside the crate-source census
# (ADR-014). The crate declares none yet; the empty list keeps the
# parent's role composition uniform across crates.
vectors_excluded_files = files()
