# Planning census (ADR-014): explicit and hand-managed — a new file
# must be listed here, or the census audit fails the build.
plan_reference_names = [
  'README.md',
  'elements-tapscript.md',
]
foreach name : plan_reference_names
  plan_files += files(name)
  plan_paths += 'plans/reference' / name
endforeach
