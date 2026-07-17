# Planning census (ADR-014): explicit and hand-managed — a new file
# must be listed here, or the census audit fails the build.
plan_root_names = [
  'README.md',
  'backlog.md',
  'roadmap.md',
]
foreach name : plan_root_names
  plan_files += files(name)
  plan_paths += 'plans' / name
endforeach

subdir('decisions')
subdir('labels')
subdir('packages')
subdir('phases')
subdir('reference')
subdir('registers')
subdir('research')
