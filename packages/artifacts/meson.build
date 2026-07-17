# Rust-source census of the artifacts crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build.
artifacts_source_names = [
  'src/bin/check-generated.rs',
  'src/bin/generate-all.rs',
  'src/lib.rs',
  'src/tests/mod.rs',
  'src/tests/weld_tests.rs',
  'src/weld.rs',
]
foreach name : artifacts_source_names
  crate_source_files += files(name)
  crate_source_paths += 'packages/artifacts' / name
endforeach
