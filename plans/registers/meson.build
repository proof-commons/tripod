# Planning census (ADR-014): explicit and hand-managed — a new file
# must be listed here, or the census audit fails the build.
plan_registers_names = [
  'README.md',
  'artifacts.md',
  'identities.md',
]
foreach name : plan_registers_names
  plan_files += files(name)
  plan_paths += 'plans/registers' / name
endforeach
