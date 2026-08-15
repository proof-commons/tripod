# Planning census (ADR-014): explicit and hand-managed — a new file must
# be listed here, or the census audit fails the build. The parent
# meson.build accumulates this list into its role group.
plans_root_files = files(
  'README.md',
  'backlog.md',
  'roadmap.md',
)

subdir('decisions')
subdir('guides')
subdir('labels')
subdir('packages')
subdir('phases')
subdir('reference')
subdir('registers')
subdir('research')
subdir('reviews')

# The planning role group, composed from the per-directory lists.
plan_files = plans_root_files + plans_decisions_files + plans_guides_files \
  + plans_labels_files + plans_packages_files + plans_phases_files \
  + plans_reference_files + plans_registers_files + plans_research_files \
  + plans_reviews_files
