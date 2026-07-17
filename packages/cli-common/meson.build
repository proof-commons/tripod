# Rust-source census of the cli-common crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build.
cli_common_source_names = [
  'src/lib.rs',
  'src/tests/mod.rs',
]
foreach name : cli_common_source_names
  crate_source_files += files(name)
  crate_source_paths += 'packages/cli-common' / name
endforeach
