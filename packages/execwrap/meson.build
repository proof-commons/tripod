# Rust-source census of the execwrap crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build.
execwrap_source_names = [
  'src/bin/execwrap.rs',
  'src/lib.rs',
  'src/tests/mod.rs',
  'src/writer.rs',
]
foreach name : execwrap_source_names
  crate_source_files += files(name)
  crate_source_paths += 'packages/execwrap' / name
endforeach
