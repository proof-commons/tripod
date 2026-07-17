# Rust-source census of the architecture crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build.
architecture_source_names = [
  'src/canonical.rs',
  'src/deployment.rs',
  'src/export.rs',
  'src/ids.rs',
  'src/lib.rs',
  'src/spec.rs',
  'src/tests/conformance_tests.rs',
  'src/tests/deployment_tests.rs',
  'src/tests/export_hash_tests.rs',
  'src/tests/mod.rs',
  'src/tests/mutation_tests.rs',
  'src/tests/validation_tests.rs',
  'src/tests/versioning_gate_tests.rs',
  'src/validate.rs',
]
foreach name : architecture_source_names
  crate_source_files += files(name)
  crate_source_paths += 'packages/architecture' / name
endforeach
