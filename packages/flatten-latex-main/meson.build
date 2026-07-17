# Rust-source census of the flatten-latex-main crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build.
flatten_latex_main_source_names = [
  'src/bin/flatten-latex-main.rs',
  'src/lib.rs',
  'src/tests/mod.rs',
]
foreach name : flatten_latex_main_source_names
  crate_source_files += files(name)
  crate_source_paths += 'packages/flatten-latex-main' / name
endforeach
