# Planning census (ADR-014): explicit and hand-managed — a new file must
# be listed here, or the census audit fails the build. The parent
# meson.build accumulates this list into its role group.
plans_packages_files = files(
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
)

subdir('errors')
plans_packages_files += plans_packages_errors_files
