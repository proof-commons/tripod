# Planning census (ADR-014): explicit and hand-managed — a new file
# must be listed here, or the census audit fails the build.
plan_decisions_names = [
  '001-typed-rust-source.md',
  '002-realization-layer.md',
  '003-tapscript-first.md',
  '004-translation-validation.md',
  '005-value-representation.md',
  '006-transaction-abi.md',
  'README.md',
]
foreach name : plan_decisions_names
  plan_files += files(name)
  plan_paths += 'plans/decisions' / name
endforeach
