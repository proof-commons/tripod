# Planning census (ADR-014): explicit and hand-managed — a new file
# must be listed here, or the census audit fails the build.
plan_packages_errors_names = [
  'README.md',
  'compiler.md',
  'labels.md',
  'linker.md',
  'realization.md',
  'release.md',
  'simplicity.md',
  'tapscript.md',
  'target-elements.md',
  'transaction.md',
  'vectors.md',
]
foreach name : plan_packages_errors_names
  plan_files += files(name)
  plan_paths += 'plans/packages/errors' / name
endforeach
