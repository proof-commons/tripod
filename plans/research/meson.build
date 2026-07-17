# Planning census (ADR-014): explicit and hand-managed — a new file
# must be listed here, or the census audit fails the build.
plan_research_names = [
  'README.md',
  'public-declassification.md',
  'settlement-layout.md',
  'state-constructor.md',
  'wide-arithmetic.md',
]
foreach name : plan_research_names
  plan_files += files(name)
  plan_paths += 'plans/research' / name
endforeach
