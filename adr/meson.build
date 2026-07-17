# ADR census (ADR-014): explicit and hand-managed — a new record must
# be listed here, or the census audit fails the build.
adr_record_names = [
  '010-command-line-output-contract.md',
  '011-toolchain-and-dependency-policy.md',
  '012-documentation-labels.md',
  '013-global-label-graph.md',
  '014-meson-lint-census-and-stamps.md',
]
foreach name : adr_record_names
  adr_files += files(name)
  adr_paths += 'adr' / name
endforeach

# Non-numbered documentation in this directory (DOC owner).
doc_files += files('README.md')
doc_paths += 'adr/README.md'
