# Rust-source census of the labels crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build.
labels_source_names = [
  'src/bin/check-labels.rs',
  'src/bin/generate-label-registers.rs',
  'src/census.rs',
  'src/check.rs',
  'src/diagnostic.rs',
  'src/label.rs',
  'src/latex.rs',
  'src/lib.rs',
  'src/markdown.rs',
  'src/owner.rs',
  'src/registry.rs',
  'src/render.rs',
  'src/repository.rs',
  'src/rust_source.rs',
  'src/source.rs',
  'src/tests.rs',
]
foreach name : labels_source_names
  crate_source_files += files(name)
  crate_source_paths += 'packages/labels' / name
endforeach

# Crate documentation (DOC owner).
doc_files += files('README.md')
doc_paths += 'packages/labels/README.md'
