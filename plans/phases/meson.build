# Planning census (ADR-014): explicit and hand-managed — a new file
# must be listed here, or the census audit fails the build.
plan_phases_names = [
  '00-baseline.md',
  '01-realization.md',
  '02-compiler.md',
  '03-target-foundation.md',
  '04-compact-ash.md',
  '05-live-transfer.md',
  '06-state-and-maturity.md',
  '07-burn-and-clear.md',
  '08-redemption.md',
  '09-requests-and-admission.md',
  '10-settlement.md',
  '11-cycle.md',
  '12-release.md',
  'README.md',
]
foreach name : plan_phases_names
  plan_files += files(name)
  plan_paths += 'plans/phases' / name
endforeach
