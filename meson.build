project(
  'tripod',
  version: '0.5.0-dev',
  meson_version: '>=1.3.0',
  subproject_dir: 'papers',
)

fs = import('fs')

# ---------------------------------------------------------------------------
# External tools required for the LaTeX build. In mock_mode the TeX
# toolchain is simulated by execwrap --mock-child, so it is not required —
# this lets the mocked Meson contract test configure without TeX on PATH.
# ---------------------------------------------------------------------------
mock_mode = get_option('mock_mode')
latexmk = find_program('latexmk', required: not mock_mode)
xelatex = find_program('xelatex', required: not mock_mode)
biber = find_program('biber', required: not mock_mode)

# ---------------------------------------------------------------------------
# Rust toolchain. Configuration never compiles the workspace: every
# required helper binary is a custom target running
# scripts/cargo-bin-sync.sh — `cargo build --release` for that
# binary plus a copy-if-changed into the declared output — so the
# consuming targets carry real dependency edges and cargo remains the
# Rust dependency tracker.
# ---------------------------------------------------------------------------
cargo = find_program('cargo', required: true)
cargo_manifest = meson.project_source_root() / 'Cargo.toml'
cargo_bin_sync = find_program('scripts/cargo-bin-sync.sh')
sh = find_program('sh')

# Git is used by the census audit and passed to document subprojects
# that derive build-time provenance metadata from the selected source
# revision. It is required for canonical publication.
git = find_program('git', required: true)

latex_default_options = [
  'git_path=' + git.full_path(),
  'mock_mode=' + mock_mode.to_string(),
]
if not mock_mode
  latex_default_options += [
    'xelatex_path=' + xelatex.full_path(),
    'biber_path=' + biber.full_path(),
    'latexmk_path=' + latexmk.full_path(),
  ]
endif

# ---------------------------------------------------------------------------
# Paper subproject. Besides the document targets it owns the attestation
# part of the lint census (meson's subproject sandbox forbids the
# parent grabbing those files itself).
# ---------------------------------------------------------------------------
attestation_proj = subproject('attestation', default_options: latex_default_options)

attestation_dep = attestation_proj.get_variable('document_dep')
attestation_pdf = attestation_proj.get_variable('install_pdf')
attestation_flat = attestation_proj.get_variable('flat_build_target')

attestation_main_file = attestation_proj.get_variable('main_tex')
attestation_section_files = attestation_proj.get_variable('attestation_section_files')

# ---------------------------------------------------------------------------
# Lint census (ADR-014). Every subject directory declares its files in
# an explicit, hand-managed meson.build list under its own name, and
# each parent composes its children's lists into the role groups. The
# census-audit target diffs the declared lists against git ls-files on
# every build, and each checker re-verifies its argument census
# against the on-disk tree, so a missed file is a build failure, never
# a silent omission.
# ---------------------------------------------------------------------------
subdir('adr')
subdir('docs')
subdir('plans')
subdir('packages')
subdir('scripts')

adr_files = adr_record_files

# The realization document owns its own R13 labels; it lives under
# docs/attestation so its edits do not perturb the paper's identity.
realization_file = attestation_realization_file

# Repository-root and repository documentation (DOC owner).
doc_files = files('AGENTS.md', 'README.md', 'SECURITY.md') \
  + adr_doc_files + packages_doc_files \
  + docs_doc_files

# Repository-relative path strings for checker argv and the census
# audit, derived from the declared file objects.
source_root = meson.project_source_root()
attestation_main_path = fs.relative_to(attestation_main_file[0], source_root)
realization_path = fs.relative_to(realization_file[0], source_root)
specification_register_path = fs.relative_to(specification_register_file[0], source_root)
realization_register_path = fs.relative_to(realization_register_file[0], source_root)
attestation_register_path = fs.relative_to(attestation_register_file[0], source_root)
model_labels_json_path = fs.relative_to(model_labels_json_file[0], source_root)

attestation_section_paths = []
foreach f : attestation_section_files
  attestation_section_paths += fs.relative_to(f, source_root)
endforeach
adr_paths = []
foreach f : adr_files
  adr_paths += fs.relative_to(f, source_root)
endforeach
plan_paths = []
foreach f : plan_files
  plan_paths += fs.relative_to(f, source_root)
endforeach
doc_paths = []
foreach f : doc_files
  doc_paths += fs.relative_to(f, source_root)
endforeach
model_source_paths = []
foreach f : model_source_files
  model_source_paths += fs.relative_to(f, source_root)
endforeach
crate_source_paths = []
foreach f : crate_source_files
  crate_source_paths += fs.relative_to(f, source_root)
endforeach
script_source_paths = []
foreach f : script_source_files
  script_source_paths += fs.relative_to(f, source_root)
endforeach
generated_paths = []
foreach f : generated_files
  generated_paths += fs.relative_to(f, source_root)
endforeach

generated_dir = fs.parent(model_labels_json_path)
artifact_generation_output_dir = get_option('artifact_generation_output_dir')
if artifact_generation_output_dir == ''
  artifact_generation_output_dir = generated_dir
endif

publication_archive_root = get_option('publication_archive_root')
if publication_archive_root == ''
  publication_archive_root = meson.project_source_root() / 'archive'
endif

# Role-tagged argument lists. Paths stay repository-relative: every
# binary resolves them against --repository-root, while the file
# objects above give ninja its dependency edges.
register_census_args = [
  '--repository-root', meson.project_source_root(),
  '--attestation-main', attestation_main_path,
  '--realization', realization_path,
]
foreach p : attestation_section_paths
  register_census_args += ['--attestation-section', p]
endforeach

scoped_census_args = register_census_args
foreach p : model_source_paths
  scoped_census_args += ['--model-source', p]
endforeach

labels_census_args = scoped_census_args + [
  '--specification-register', specification_register_path,
  '--realization-register', realization_register_path,
  '--attestation-register', attestation_register_path,
  '--model-labels-json', model_labels_json_path,
]
foreach p : adr_paths
  labels_census_args += ['--adr', p]
endforeach
foreach p : plan_paths
  labels_census_args += ['--plan', p]
endforeach
foreach p : doc_paths
  labels_census_args += ['--doc', p]
endforeach
foreach p : crate_source_paths
  labels_census_args += ['--crate-source', p]
endforeach
foreach p : script_source_paths
  labels_census_args += ['--script-source', p]
endforeach

census_paths = [attestation_main_path, realization_path, specification_register_path,
                realization_register_path, attestation_register_path,
                model_labels_json_path] \
               + attestation_section_paths + adr_paths + plan_paths + doc_paths \
               + model_source_paths + crate_source_paths + script_source_paths \
               + generated_paths

# Tracked paths that are categorically never lint subjects: dotfiles,
# archives, shell helper scripts, build and package definitions,
# licences, and binary or data blobs. Same-typed exceptions (a stray
# .tex or .rs that is deliberately not a subject) are NOT listed here —
# they live in their directory's explicit exclusion list.
#
# ADR-023 narrowed the script-tree entry from every path under scripts/
# to the shell sources there: the Python sources are label subjects,
# declared in scripts/meson.build. A file added under scripts/ that is
# neither Python nor shell therefore fails the census audit until it is
# classified, which is the intended effect.
#
# The tabular data extension joins the same categorical entry as the
# other data blobs: a rule table read by a checker is an input to a
# program, not prose a label could be attached to, and giving it a
# census list entry would say the opposite.
census_exclude_pattern = '(^|/)[.]|^archive/|^scripts/[^/]*[.]sh$|^LICENSE-' \
  + '|(^|/)meson[.](build|options)$|(^|/)Cargo[.]toml$|[.](bib|tsv|txt|zip)$'

census_excluded_files = packages_excluded_files \
  + attestation_proj.get_variable('attestation_excluded_files')
census_excluded_args = []
foreach f : census_excluded_files
  census_excluded_args += ['--excluded', fs.relative_to(f, source_root)]
endforeach

# ---------------------------------------------------------------------------
# First-party checker and generator binaries as real build targets
# (same idiom as the paper subproject's helper binaries).
# ---------------------------------------------------------------------------
check_labels_bin = custom_target(
  'cargo-check-labels',
  output: 'check-labels-bin',
  command: [
    cargo_bin_sync, cargo, cargo_manifest,
    'tripod-labels', 'check-labels', '@OUTPUT@',
  ],
  console: true,
  build_always_stale: true,
)

check_generated_bin = custom_target(
  'cargo-check-generated',
  output: 'check-generated-bin',
  command: [
    cargo_bin_sync, cargo, cargo_manifest,
    'tripod-artifacts', 'check-generated', '@OUTPUT@',
  ],
  console: true,
  build_always_stale: true,
)

generate_registers_bin = custom_target(
  'cargo-generate-label-registers',
  output: 'generate-label-registers-bin',
  command: [
    cargo_bin_sync, cargo, cargo_manifest,
    'tripod-labels', 'generate-label-registers', '@OUTPUT@',
  ],
  console: true,
  build_always_stale: true,
)

generate_attestation_bin = custom_target(
  'cargo-generate-attestation-register',
  output: 'generate-attestation-register-bin',
  command: [
    cargo_bin_sync, cargo, cargo_manifest,
    'tripod-labels', 'generate-attestation-register', '@OUTPUT@',
  ],
  console: true,
  build_always_stale: true,
)

check_plans_bin = custom_target(
  'cargo-check-plans',
  output: 'check-plans-bin',
  command: [
    cargo_bin_sync, cargo, cargo_manifest,
    'tripod-labels', 'check-plans', '@OUTPUT@',
  ],
  console: true,
  build_always_stale: true,
)

check_forbidden_text_bin = custom_target(
  'cargo-check-forbidden-text',
  output: 'check-forbidden-text-bin',
  command: [
    cargo_bin_sync, cargo, cargo_manifest,
    'tripod-labels', 'check-forbidden-text', '@OUTPUT@',
  ],
  console: true,
  build_always_stale: true,
)

check_hash_citations_bin = custom_target(
  'cargo-check-hash-citations',
  output: 'check-hash-citations-bin',
  command: [
    cargo_bin_sync, cargo, cargo_manifest,
    'tripod-labels', 'check-hash-citations', '@OUTPUT@',
  ],
  console: true,
  build_always_stale: true,
)

census_audit_bin = custom_target(
  'cargo-census-audit',
  output: 'census-audit-bin',
  command: [
    cargo_bin_sync, cargo, cargo_manifest,
    'tripod-labels', 'census-audit', '@OUTPUT@',
  ],
  console: true,
  build_always_stale: true,
)

# Census audit (ADR-014): the binary invokes git ls-files itself, so
# the weld is fresh on every build — a file tracked without a census
# list entry fails immediately, not at the next reconfigure. No lint
# stamp depends on this target: its stamp is re-touched on every
# successful run (the target is unconditional), and an edge would
# needlessly dirty the lint suites each build; a failing audit fails
# the build by itself. The checker runs in build mode: it publishes its
# ADR-010 JSON result as an explicit report output (compare-if-changed)
# and touches the success stamp only after the report is written, so no
# shell stdout redirection sits on the correctness boundary.
census_audit = custom_target(
  'census-audit',
  output: ['census.ok', 'census-audit.json'],
  command: [census_audit_bin,
    '--repository-root', source_root,
    '--git', git.full_path(),
    '--exclude-pattern', census_exclude_pattern,
    '--report', '@OUTPUT1@',
    '--stamp', '@OUTPUT0@',
  ] + census_excluded_args + census_paths,
  build_always_stale: true,
  build_by_default: true,
  console: true,
)

generate_all_bin = custom_target(
  'cargo-generate-all',
  output: 'generate-all-bin',
  command: [
    cargo_bin_sync, cargo, cargo_manifest,
    'tripod-artifacts', 'generate-all', '@OUTPUT@',
  ],
  console: true,
  build_always_stale: true,
)

# ---------------------------------------------------------------------------
# Lint suites as stamp targets (ADR-014): inputs are the suite's census
# slice plus the checker binary, and outputs are a build-dir stamp the
# checker touches on success plus the checker JSON report. A rerun on
# an unchanged tree executes nothing.
# ---------------------------------------------------------------------------
register_census_inputs = attestation_main_file + attestation_section_files + realization_file
scoped_census_inputs = register_census_inputs + model_source_files

labels_stamp = custom_target(
  'labels-check',
  input: scoped_census_inputs + adr_files + plan_files + doc_files
         + crate_source_files + specification_register_file + realization_register_file
         + attestation_register_file + model_labels_json_file,
  output: ['labels.ok', 'labels-check.json'],
  command: [check_labels_bin] + labels_census_args + [
    '--report', '@OUTPUT1@',
    '--stamp', '@OUTPUT0@',
  ],
  build_by_default: true,
  console: true,
)

generated_stamp = custom_target(
  'generated-check',
  input: scoped_census_inputs + generated_files + model_labels_json_file,
  output: ['generated.ok', 'generated-check.json'],
  command: [check_generated_bin] + scoped_census_args + [
    '--generated-dir', generated_dir,
    '--report', '@OUTPUT1@',
    '--stamp', '@OUTPUT0@',
  ],
  build_by_default: true,
  console: true,
)

# The plan-structure lane covers everything under adr/ and plans/,
# including the non-numbered ADR README (DOC owner) and the generated
# registers.
plans_lane_files = []
plans_lane_paths = []
foreach f : adr_files + plan_files + doc_files
  p = fs.relative_to(f, source_root)
  if p.startswith('adr/') or p.startswith('plans/')
    plans_lane_files += f
    plans_lane_paths += p
  endif
endforeach
plans_lane_files += specification_register_file + realization_register_file \
  + attestation_register_file
plans_lane_paths += [specification_register_path, realization_register_path,
                     attestation_register_path]
plans_census_args = []
foreach p : plans_lane_paths
  plans_census_args += ['--subject', p]
endforeach

plans_stamp = custom_target(
  'plans-check',
  input: plans_lane_files,
  output: ['plans.ok', 'plans-check.json'],
  command: [check_plans_bin,
    '--repository-root', meson.project_source_root(),
  ] + plans_census_args + [
    '--report', '@OUTPUT1@',
    '--stamp', '@OUTPUT0@',
  ],
  build_by_default: true,
  console: true,
)

# Forbidden-text audit (ADR-014 always-fresh repository audit): the
# binary invokes git grep over the complete tracked set on every build,
# so it is build_always_stale rather than tracking a fixed input list.
# It runs in build mode — an explicit JSON report plus a success stamp
# touched only after the report is written — so its ADR-010 diagnostics
# never sit behind shell stdout redirection.
forbidden_text_stamp = custom_target(
  'forbidden-text-check',
  output: ['forbidden-text.ok', 'forbidden-text.json'],
  command: [
    check_forbidden_text_bin,
    '--repository-root', source_root,
    '--git', git.full_path(),
    '--report', '@OUTPUT1@',
    '--stamp', '@OUTPUT0@',
  ],
  build_by_default: true,
  build_always_stale: true,
  console: true,
)

# Hash-citation audit (ADR-014 always-fresh repository audit, ADR-021
# amendment): the binary asks git for the complete tracked set on every
# build, so it is build_always_stale for the same reason the
# forbidden-text audit is — a value committed into a new file must be
# adjudicated at once, not at the next reconfigure. The families table
# is passed by path like every other checker subject (ADR-014 role-tagged
# arguments): the binary resolves no repository path from its own
# location.
hash_citations_stamp = custom_target(
  'hash-citations-check',
  output: ['hash-citations.ok', 'hash-citations.json'],
  command: [
    check_hash_citations_bin,
    '--repository-root', source_root,
    '--git', git.full_path(),
    '--families', source_root / 'lint' / 'hash-citation-families.tsv',
    '--report', '@OUTPUT1@',
    '--stamp', '@OUTPUT0@',
  ],
  build_by_default: true,
  build_always_stale: true,
  console: true,
)

# The census audit is part of the lint alias: it is the single owner of
# the tracked-entry mode rule, and it covers the complete tracked set
# rather than the lint subject slice, so a request to lint the
# repository that skipped it would leave tracked symlinks and gitlinks
# under categorically excluded paths unaudited (ADR-014). Membership
# here is what makes `meson compile -C build lint` — and therefore the
# `meson-mock-contract` lane — actually run the audit; an
# explicit Ninja target never pulls in unrelated build_by_default ones.
alias_target(
  'lint',
  census_audit,
  labels_stamp,
  generated_stamp,
  plans_stamp,
  forbidden_text_stamp,
  hash_citations_stamp,
)

# ---------------------------------------------------------------------------
# Generators: committed in-tree publications cannot be declared
# outputs, so a shell wrapper touches the build-dir stamp after the
# generator succeeds (ADR-014 generator-stamp rule). On-demand targets
# are always stale so an explicit repair request reruns the generator
# even when the stamp is newer than a corrupted committed publication.
# This stamp is an always-stale marker, not a freshness oracle, so plain
# shell `touch` is correct here — it is deliberately outside the strict
# empty-stamp refusal that governs checker `--stamp` outputs in
# cli_common::touch_stamp (reviewed as F4-003).
# ---------------------------------------------------------------------------
generator_wrap = [
  sh, '-c',
  'stamp="$1"; program="$2"; shift 2; case "$program" in */*) "$program" "$@" ;; *) "./$program" "$@" ;; esac && touch "$stamp"',
  'sh', '@OUTPUT@',
]

generate_registers_target = custom_target(
  'run-generate-label-registers',
  input: register_census_inputs,
  output: 'generate-label-registers.ok',
  command: generator_wrap + [generate_registers_bin] + register_census_args + [
    '--specification-register-output', specification_register_path,
    '--realization-register-output', realization_register_path,
  ],
  build_by_default: false,
  build_always_stale: true,
  console: true,
)
alias_target('generate-label-registers', generate_registers_target)

# The companion register derives from the whole corpus, not the scoped
# register slice: its evidence rows carry a mint census, which answers
# to every owner. Its inputs are therefore the label census.
generate_attestation_target = custom_target(
  'run-generate-attestation-register',
  input: scoped_census_inputs + adr_files + plan_files + doc_files
         + crate_source_files + model_labels_json_file,
  output: 'generate-attestation-register.ok',
  command: generator_wrap + [generate_attestation_bin] + labels_census_args,
  build_by_default: false,
  build_always_stale: true,
  console: true,
)
alias_target('generate-attestation-register', generate_attestation_target)

generate_artifacts_target = custom_target(
  'run-generate-artifacts',
  input: scoped_census_inputs,
  output: 'generate-artifacts.ok',
  command: generator_wrap + [generate_all_bin] + scoped_census_args + [
    '--output', artifact_generation_output_dir,
  ],
  build_by_default: false,
  build_always_stale: true,
  console: true,
)
alias_target('generate-artifacts', generate_artifacts_target)

# ---------------------------------------------------------------------------
# Archive exports. `install_pdf` is the canonical build-dir artifact
# consumed by `meson install`; the mirror targets repair the PDF under
# archive/rendered/ and the flattened .tex under archive/flattened/,
# gated on the label-check stamp (ADR-014): a publication must not
# embed an unlinted label graph. The plain in-build PDF compile stays
# independent for fast draft iteration.
# ---------------------------------------------------------------------------
sync_publication = find_program('scripts/sync-publication.sh')

attestation_mirror = custom_target(
  'mirror-attestation-pdf',
  input: attestation_pdf,
  output: 'attestation-pdf.mirrored.stamp',
  command: [
    sync_publication,
    '@INPUT@',
    publication_archive_root / 'rendered' / fs.name(attestation_pdf.full_path()),
    '@OUTPUT@',
  ],
  depends: labels_stamp,
  build_by_default: true,
  build_always_stale: true,
  console: false,
)

attestation_flatten = custom_target(
  'flatten-attestation-mirror',
  input: attestation_flat,
  output: 'attestation-flat.mirrored.stamp',
  command: [
    sync_publication,
    '@INPUT@',
    publication_archive_root / 'flattened' / fs.name(attestation_flat.full_path()),
    '@OUTPUT@',
  ],
  depends: labels_stamp,
  build_by_default: true,
  build_always_stale: true,
  console: false,
)

alias_target('attestation', attestation_mirror, attestation_flatten)
alias_target('flatten-attestation', attestation_flatten)
alias_target('papers', attestation_mirror, attestation_flatten)

# ---------------------------------------------------------------------------
# Meson test surface. The cargo-global lanes stay cargo-tracked
# (ADR-014): cargo is their dependency tracker and resolves versions
# from Cargo.toml (ADR-011). The lint suites are thin wrappers over
# the stamp targets above — `meson test` builds the stamps (running
# only what changed) and then merely asserts they exist.
# ---------------------------------------------------------------------------
# Budget: 0:01, tolerance 0:05. rustfmt parses, it does not compile.
test(
  'cargo-fmt',
  cargo,
  args: ['fmt', '--all', '--check', '--manifest-path', cargo_manifest],
  suite: 'lint',
  timeout: 120,
)

stamp_probe = ['-c', 'test -f "$1"', 'sh']

# clippy and the test suite compile the whole workspace on a cold cache, so
# they need far more than meson's default 30-second test timeout. These targets
# are deliberately always-stale: Meson does not model Cargo's complete input
# graph here, so the command must run and let Cargo decide what is fresh. Without
# this, an old cargo-*.ok stamp could satisfy `meson test` after Rust sources,
# manifests, or Cargo's own dependency graph changed. Like generator_wrap, the
# cargo-*.ok stamp is an always-stale marker, so plain shell `touch` is correct
# and stays outside cli_common::touch_stamp's strict empty-stamp rule (F4-003).
cargo_quiet_stamp_wrap = [
  sh, '-c',
  'stamp="$1"; log="$2"; shift 2; if CARGO_TERM_PROGRESS_WHEN=never "$@" > "$log" 2>&1; then touch "$stamp"; else cat "$log" >&2; exit 1; fi',
  'sh', '@OUTPUT0@', '@OUTPUT1@',
]

cargo_clippy_stamp = custom_target(
  'cargo-clippy-check',
  output: ['cargo-clippy.ok', 'cargo-clippy.log'],
  command: cargo_quiet_stamp_wrap + [
    cargo,
    'clippy',
    '--workspace',
    '--all-targets',
    '--manifest-path', cargo_manifest,
    '--',
    '-D', 'warnings',
  ],
  build_by_default: false,
  build_always_stale: true,
  console: true,
)

# Budget: 0:20 warm, tolerance 6:00 cold. The stamp target compiles the whole
# workspace with all targets on a cold cache; warm it is a cargo freshness
# check. Both figures are the same lane, and the spread is the cache. The
# test() itself only probes the stamp, so its reported duration is the probe
# (0:00.2 measured) and the compile is attributed to the build phase -- which
# is where it belongs, and is why this lane's budget is not read off the test
# report alone.
test(
  'cargo-clippy',
  sh,
  args: stamp_probe + [cargo_clippy_stamp[0].full_path()],
  depends: cargo_clippy_stamp,
  suite: 'lint',
  timeout: 60,
)

# ---------------------------------------------------------------------------
# Per-package test lanes (CI-002). The gate runs no workspace-level
# `cargo test`: each workspace member is its own Meson test, so a
# duration and a failure both name the package they belong to, and
# `meson test --suite cargo-debug` is the whole Rust test surface for
# that profile. Both profiles keep a full set of lanes; the release
# duplicate is retained deliberately, and the per-package data is what
# a later ruling on its value will be argued from.
#
# The list is the thirteen [workspace].members of Cargo.toml, spelled
# out in the census discipline of ADR-014 rather than derived: each
# entry pairs the package directory (the lane's short name) with the
# published package name that `cargo test -p` takes, so a rename fails
# here loudly instead of silently dropping a package's tests. A member
# added to Cargo.toml without an entry here has no lane -- the
# per-package test-lane census audit below is what refuses that.
#
# That audit is the same weld the ADR-014 census uses, applied to the
# manifest: the declared lane list is diffed against the members list
# read out of Cargo.toml at configuration time, in both directions, and
# a mismatch is a hard configuration error rather than a quietly
# missing lane.
cargo_test_packages = [
  ['architecture', 'tripod-architecture'],
  ['artifacts', 'tripod-artifacts'],
  ['cli-common', 'cli-common'],
  ['compiler', 'tripod-compiler'],
  ['document-stamps', 'tripod-document-stamps'],
  ['execwrap', 'execwrap'],
  ['flatten-latex-main', 'flatten-latex-main'],
  ['labels', 'tripod-labels'],
  ['linker', 'tripod-linker'],
  ['model', 'tripod-model'],
  ['realization', 'tripod-realization'],
  ['tapscript', 'tripod-tapscript'],
  ['target-elements', 'tripod-target-elements'],
  ['target-elements-conformance', 'tripod-target-elements-conformance'],
  ['transaction', 'tripod-transaction'],
  ['vectors', 'tripod-vectors'],
]

cargo_test_lane_dirs = []
foreach entry : cargo_test_packages
  cargo_test_lane_dirs += entry[0]
endforeach

cargo_manifest_members = []
foreach line : fs.read('Cargo.toml').split('members = [')[1].split(']')[0].split('\n')
  member = line.strip().strip(',').strip('"')
  if member != ''
    cargo_manifest_members += member
  endif
endforeach

foreach member : cargo_manifest_members
  if not cargo_test_lane_dirs.contains(member.split('packages/')[-1])
    error('workspace member ' + member
      + ' has no per-package test lane; add it to cargo_test_packages')
  endif
endforeach
foreach dir : cargo_test_lane_dirs
  if not cargo_manifest_members.contains('packages/' + dir)
    error('per-package test lane ' + dir
      + ' is not a Cargo.toml [workspace].members entry')
  endif
endforeach

cargo_test_json = find_program('scripts/cargo-test-json.sh')

# Warm-up: one workspace-wide `cargo build --tests` per profile, which
# every per-package lane of that profile depends on. Cargo serializes
# concurrent invocations on a lock over the shared target directory, so
# thirteen lanes that each had to compile would spend most of their wall
# time queued behind that lock and their per-package durations would
# measure the queue rather than the package. Building once and running
# many leaves each lane holding the lock only briefly, which is what
# makes the per-package figures attributable at all.
#
# Measured on a 14-core host, wall clock, the debug lanes (CI-002):
#
#   warm-up build then serial lanes (--num-processes 1)   0:52 + 8:32
#   warm-up build then parallel lanes (meson's default)   0:52 + 5:00
#   no warm-up, parallel lanes, cold target directory            6:53
#
# Two things follow. Parallel is faster, so the lanes really do overlap:
# cargo's lock over the shared target directory is held only briefly
# once the binaries are built, and the serialization it could have
# forced does not materialize. The gain is bounded from below by the
# slowest single package -- target-elements-conformance alone is about
# 250s serial and 300s while the other lanes share the cores -- so it is
# 1.7x, not 13x. And the third row is why the warm-up exists at all:
# without it every lane's cargo has to build, it is slower in total, and
# worse, each lane's duration then measures how long that lane waited
# for the build lock rather than how long its tests took.
#
# What the warm-up does NOT cover is documentation tests: `cargo build
# --tests` does not compile them, so rustdoc compiles each package's
# doctests inside its own lane, holding cargo's lock while it does. In
# debug that cost is small. In release it dominates: a full gate run
# measured 7:04 of warm-up and 10:31 of lanes, with individual packages
# reporting ten-minute durations against a tenth of a second of actual
# test execution and cargo's own "Blocking waiting for file lock" lines
# in the logs. The release per-package durations are therefore honest
# wall times but poor attributions -- they are mostly queueing -- and
# that is a known residual of this design, not a measurement to read as
# test cost.
cargo_test_warmup = {}
foreach profile : ['debug', 'release']
  warmup_args = [
    cargo, 'build', '--tests', '--workspace',
    '--manifest-path', cargo_manifest,
  ]
  if profile == 'release'
    warmup_args += ['--release']
  endif
  warmup = custom_target(
    'run-cargo-test-warmup-' + profile,
    output: ['cargo-test-warmup-' + profile + '.ok',
             'cargo-test-warmup-' + profile + '.log'],
    command: cargo_quiet_stamp_wrap + warmup_args,
    build_by_default: false,
    build_always_stale: true,
    console: true,
  )
  alias_target('cargo-test-warmup-' + profile, warmup)
  cargo_test_warmup += {profile: warmup}
endforeach

# The lanes themselves. Each runs the package's complete test set --
# unit, integration, and documentation tests, exactly what
# `cargo test -p <package>` ran before -- and captures libtest's JSON
# event stream under the build tree, from which scripts/ci-timing-report.py
# reads per-test durations. Meson times a lane; only libtest can say
# which test inside it was slow, which is the attribution CI-002 split
# the suite per package to obtain. is_parallel is pinned rather than
# left at its default: the measurement behind the value is directly
# above.
#
# Budget (CI-005), stated per suite, which is the unit a cadence
# decision actually chooses. Per lane the spread is too wide to budget:
# target-elements-conformance owns roughly a sixth of the debug suite by
# itself. Warm cache, sixteen lanes: debug 5:00, tolerance 8:00; release
# 10:30, tolerance 15:00. Cold cache, measured 2026-08-20 on the 14-core
# reference host: debug 15:45, release 30:30, tolerance 1.5x each.
#
# The cold and warm figures differ by 3x, so a run that does not say
# which cache it had has not reported its duration. The release suite is
# the whole gate's cost -- 30:30 of a 33:20 test phase -- and most of it
# is the doctest queueing described above rather than test execution.
cargo_test_stream_root = meson.project_build_root() / 'cargo-test-json'

foreach profile : ['debug', 'release']
  foreach entry : cargo_test_packages
    test(
      'cargo-test-' + profile + '-' + entry[0],
      cargo_test_json,
      args: [
        cargo.full_path(),
        cargo_manifest,
        entry[1],
        profile,
        cargo_test_stream_root / profile / (entry[0] + '.json'),
      ],
      depends: cargo_test_warmup[profile],
      suite: 'cargo-' + profile,
      timeout: 1800,
      is_parallel: true,
    )
  endforeach
endforeach

# Budget: 0:02, tolerance 0:10. One `git ls-files` welded against the
# hand-managed per-directory lists.
test(
  'census-audit',
  sh,
  args: stamp_probe + [census_audit[0].full_path()],
  depends: census_audit,
  suite: 'lint',
  timeout: 60,
)

# Budget: 0:02, tolerance 0:10. The stamp probe is instant; the cost is the
# checker target behind it, which reruns only when its census slice changed.
test(
  'check-generated',
  sh,
  args: stamp_probe + [generated_stamp[0].full_path()],
  depends: generated_stamp,
  suite: 'generated',
  timeout: 60,
)

# Budget: 0:05, tolerance 0:20. The label graph is the largest census slice.
test(
  'labels-check',
  sh,
  args: stamp_probe + [labels_stamp[0].full_path()],
  depends: labels_stamp,
  suite: ['documentation', 'generated'],
  timeout: 60,
)

# Budget: 0:03, tolerance 0:15.
test(
  'plans-check',
  sh,
  args: stamp_probe + [plans_stamp[0].full_path()],
  depends: plans_stamp,
  suite: 'documentation',
  timeout: 60,
)

# Budget: 0:03, tolerance 0:15. Always stale: it runs git grep over the
# complete tracked set on every build rather than tracking an input list.
test(
  'forbidden-text-check',
  sh,
  args: stamp_probe + [forbidden_text_stamp[0].full_path()],
  depends: forbidden_text_stamp,
  suite: 'lint',
  timeout: 60,
)

# Budget: 0:03, tolerance 0:15. Always stale for the same reason as the
# check above: it lists the tracked set itself on every build rather than
# declaring an input list that a new file would silently fall outside.
test(
  'hash-citations-check',
  sh,
  args: stamp_probe + [hash_citations_stamp[0].full_path()],
  depends: hash_citations_stamp,
  suite: 'lint',
  timeout: 60,
)

# Real-derivation integration test (cargo + git, no TeX): proves the
# Git-derived identity scopes that the mocked contract test cannot. The
# heavier Meson graph properties (mirror/generator repair, restat, failure
# propagation) are covered by scripts/test-meson-mock.sh, which is the
# `meson-mock-contract` test declared below (CI-009 re-homed it here from the
# retired Python driver).
# Publication-mode freshness contract for the two shell publication
# paths (R2-N04): compare-if-changed equality is bytes AND required
# mode, so a byte-current destination with a wrong mode is repaired
# rather than skipped forever. Uses a mock cargo, so no Rust toolchain
# or TeX is involved.
publication_mode_test = find_program('scripts/test-publication-mode.sh')

# Budget: 0:02, tolerance 0:10. Mock cargo, no toolchain involved.
test(
  'publication-mode-freshness',
  publication_mode_test,
  args: [meson.project_source_root()],
  suite: 'integration',
  timeout: 120,
)

attestation_stamps_test = find_program('scripts/test-attestation-stamps.sh')

# Budget: 1:00, tolerance 3:00. Real cargo and git, no TeX; pinned serial
# because it drives cargo against the shared target directory.
test(
  'attestation-stamps-scopes',
  attestation_stamps_test,
  args: [meson.project_source_root()],
  suite: 'integration',
  timeout: 600,
  is_parallel: false,
)

# ---------------------------------------------------------------------------
# Gate lanes migrated from the retired Python driver (CI-009).
#
# The driver declared eleven lanes and wall-timed each one itself. Seven of
# them were already meson tests and it merely invoked them; the four below
# were the remainder, and they are re-homed here unchanged in what they
# prove. Nothing about a lane's subject moved: the argv, the scripts, and the
# exit-status meanings are the ones the driver used. What moved is who
# measures and who reports, and that is the point -- meson's harness times
# every test, writes the durations into meson-logs/testlog.json, and has a
# first-class SKIP status, so the driver's parallel implementations of all
# three are dead weight rather than a second opinion.
#
# Budgets (CI-005, the wall-time ruling). Each test carries its expected
# duration and tolerance beside the declaration. A lane outside its budget is
# a finding to report, not a cost to absorb; the figures are measurements on
# the 14-core reference host, not estimates.
# ---------------------------------------------------------------------------

python3 = find_program('python3')

# Preflight in the driver, an ordinary lane here. It refuses a tracked path
# that falls outside the safe checker-argv grammar (SR3-05), so a path that
# would need quoting can never reach a checker's argv. census-args.sh stays
# the single definition of the grammar: this sources it exactly as the driver
# did rather than restating the rule.
# Budget: 0:01, tolerance 0:02. It is one `git ls-files` and a pattern match.
test(
  'tracked-path-argv-audit',
  sh,
  args: ['-c', '. ./scripts/census-args.sh && audit_tracked_paths'],
  workdir: meson.project_source_root(),
  suite: 'lint',
  timeout: 120,
)

# The Python test surface CI-007 found missing, arriving as a meson test by
# the CI-009 ruling rather than as a driver registry entry. Node-free by
# construction: it imports the executor adapter and exercises its script-error
# classification as pure string functions, so it needs no node, no network,
# and no Rust or TeX toolchain, and it belongs in the ordinary suite rather
# than behind the target_native_executor option.
# Budget: 0:00.2, tolerance 0:02. It is an import and 112 assertions.
test(
  'executor-classification',
  python3,
  args: [meson.project_source_root() / 'scripts' / 'test-executor-classification.py'],
  suite: 'lint',
  timeout: 120,
)

gate_cargo_audit = find_program('scripts/gate-cargo-audit.sh')

# The advisory lane. Externally provisioned by ADR-011, so the wrapper exits
# 77 and meson reports SKIP when cargo-audit is absent -- the harness's own
# status, rather than a driver's private vocabulary. The wrapper locates and
# executes the advisory binary directly instead of going through `cargo
# audit`, so no caller has to put ~/.cargo/bin on PATH to satisfy this lane
# and thereby put rustup's cargo in front of the configured toolchain; see
# the script for the failure that discipline exists to prevent.
# Budget: 0:03 when provisioned, tolerance 0:15 (measured 0:02; it fetches
# nothing, the advisory database is whatever the operator last synced). A
# skip is instant.
test(
  'cargo-audit',
  gate_cargo_audit,
  args: [meson.project_source_root()],
  suite: 'lint',
  timeout: 600,
)

meson_mock_test = find_program('scripts/test-meson-mock.sh')

# The mocked Meson contract lane: the production Meson graph with the TeX
# toolchain simulated, covering the ADR-014 census wiring, report and stamp
# edges, generator and publication repair, no-op restat behaviour, and
# render-failure propagation.
#
# It is a nested Meson build, which is why the comment above the
# publication-mode test used to say it ran from the shell driver "rather than
# as a nested Meson test". That reservation was about the DRIVER's ordering,
# not about a technical bar: the nested build configures its own directory
# under build/mocks, shares no state with the outer build, and meson imposes
# nothing on a test that forbids it. It is pinned serial because it drives
# cargo, and cargo serializes on the lock over the shared target directory:
# run in parallel with the package lanes its duration would measure the
# queue.
#
# The driver made this lane skippable for local convenience, with
# CI_REQUIRE_MESON=1 to make a skip fatal. Both are now moot -- a meson test
# cannot run without meson -- so the lane is unconditional, which is the
# stronger of the two behaviours the toggle chose between.
# Budget: 0:15 warm, tolerance 0:45; 2:15 cold, tolerance 4:00. Both measured
# 2026-08-20 (0:09.8 and 2:11.3). The spread is the helper binaries: it syncs
# them through cargo, which is instant against a warm target directory and a
# full release build against a cold one. Its own work -- configuring a Meson
# build and running the repair, restat, and render-failure regressions -- is
# the warm figure.
test(
  'meson-mock-contract',
  meson_mock_test,
  args: [meson.project_source_root()],
  suite: 'integration',
  timeout: 1800,
  is_parallel: false,
)

gate_clean_tree = find_program('scripts/gate-clean-tree.sh')

# The driver's last lane, and the one that makes "a clean checkout passes
# without modifying tracked files" a checked claim rather than an assertion.
# The negative priority schedules it after the parallel work; see the script
# for why that is a preference rather than a guarantee, and why nothing in
# this suite races it.
# Budget: 0:01, tolerance 0:05. One `git status` over the tracked set.
test(
  'clean-tree',
  gate_clean_tree,
  args: [meson.project_source_root()],
  suite: 'lint',
  timeout: 120,
  is_parallel: false,
  priority: -100,
)

# ---------------------------------------------------------------------------
# Target-native primitive conformance (Guide 9): a NON-DEFAULT lane.
#
# The executor is external code the operator selects explicitly, and
# selecting it grants execution authority (ADR-015). It is therefore not
# a configured dependency of the ordinary build: with no
# `target_native_executor` option the checker target is not defined at
# all, nothing runs it, and ordinary CI claims no target-native evidence
# — a skipped native lane leaves the Guide-9 evidence incomplete even
# when every other lane is green.
#
# No credential appears in this command, and none may be added: the
# checker defines no credential argument, and an executor that must
# authenticate to a node establishes that boundary outside this process
# (ADR-015, Guide-9 §17.6).
#
# The build system makes no trust declaration on the caller's behalf
# (`G11-R07`). Selecting an executor path says which program to run; it
# does not say what that program is, and the class option therefore
# defaults to `unselected` and fails configuration rather than resolving
# to the answer that would make the run evidence. A caller who wants a
# gate must ask for one in as many words.
target_native_executor = get_option('target_native_executor')
target_native_executor_class = get_option('target_native_executor_class')
target_native_intended_tip = get_option('target_native_intended_tip')
target_native_upstream_base = get_option('target_native_upstream_base')
target_native_local_topics = get_option('target_native_local_topics')

if target_native_executor != '' and target_native_executor_class == 'unselected'
  error(
    'target_native_executor names a program but target_native_executor_class is '
    + '"unselected". The build system does not classify an executor for you: pass '
    + '-Dtarget_native_executor_class=mock for a run that can never be evidence, or '
    + '-Dtarget_native_executor_class=reviewed-non-mock to declare the reviewed runner '
    + 'and supply its ADR-018 provenance.',
  )
endif

# A reviewed nonmock target is a target configured to produce evidence,
# so every value the evidence gate needs is required at configuration
# time rather than discovered when the command refuses hours later.
if target_native_executor != '' and target_native_executor_class == 'reviewed-non-mock'
  foreach required : [
    ['target_native_network_id', get_option('target_native_network_id')],
    ['target_native_genesis_id', get_option('target_native_genesis_id')],
    ['target_native_intended_tip', target_native_intended_tip],
    ['target_native_upstream_base', target_native_upstream_base],
  ]
    if required[1] == ''
      error(
        'a reviewed-non-mock target-native executor requires an explicit '
        + required[0]
        + '. An evidence run states the chain it ran on and the source its binary was '
        + 'built from (ADR-018); the build system supplies neither by default.',
      )
    endif
  endforeach
endif

if target_native_executor != ''
  # Assembled once and shared by the primitive and prototype lanes, so
  # the two cannot drift into declaring different provenance for one
  # executable. The provenance arguments appear only for a reviewed
  # nonmock run: the commands refuse them for a mock, since no
  # expectation makes a mock evidence.
  target_native_common_args = [
    '--executor', target_native_executor,
    '--executor-class', target_native_executor_class,
    '--network-id', get_option('target_native_network_id'),
    '--genesis-id', get_option('target_native_genesis_id'),
  ]
  if target_native_executor_class == 'reviewed-non-mock'
    target_native_common_args += [
      '--intended-tip', target_native_intended_tip,
      '--upstream-base', target_native_upstream_base,
    ]
    foreach topic : target_native_local_topics
      target_native_common_args += ['--local-topic', topic]
    endforeach
  endif

  check_target_native_bin = custom_target(
    'cargo-check-target-elements-native',
    output: 'check-target-elements-native-bin',
    command: [
      cargo_bin_sync, cargo, cargo_manifest,
      'tripod-target-elements-conformance',
      'check-target-elements-native', '@OUTPUT@',
    ],
    console: true,
    build_always_stale: true,
  )

  # The report is a build-directory evidence asset, not a tracked
  # generated artifact: it records what one executor observed on one
  # host, and nothing in the repository is derived from it.
  target_native_check = custom_target(
    'target-elements-native',
    output: ['target-elements-native.ok', 'target-elements-native.json'],
    command: [check_target_native_bin] + target_native_common_args + [
      '--report', '@OUTPUT1@',
      '--stamp', '@OUTPUT0@',
      # Kept beside the report, and for the same reason: what one
      # executor said about one run on one host. One directory per
      # target, so two runs cannot append into each other's record.
      '--executor-diagnostics-directory',
      meson.current_build_dir() / 'target-elements-native-diagnostics',
    ],
    build_always_stale: true,
    build_by_default: false,
    console: true,
  )

  alias_target('target-elements-native-check', target_native_check)

  # The compound-prototype lanes, under the same option and the same
  # reasoning: the executor is external code the operator selects, so
  # nothing here is a configured dependency of the ordinary build.
  #
  # One target per relation rather than one target with a flag. A run
  # answers one relation, its report carries that relation's own claim
  # census and its own completeness status, and two relations sharing a
  # build target would share one report asset — which is the outcome the
  # separate report roles exist to prevent (Guide-10 section 15.1).
  check_target_prototypes_bin = custom_target(
    'cargo-check-target-elements-prototypes',
    output: 'check-target-elements-prototypes-bin',
    command: [
      cargo_bin_sync, cargo, cargo_manifest,
      'tripod-target-elements-conformance',
      'check-target-elements-prototypes', '@OUTPUT@',
    ],
    console: true,
    build_always_stale: true,
  )

  target_prototype_checks = []
  foreach relation : ['constructor-continuity', 'wide-floor']
    target_prototype_checks += custom_target(
      'target-elements-prototype-' + relation,
      output: [
        'target-elements-prototype-' + relation + '.ok',
        'target-elements-prototype-' + relation + '.json',
      ],
      command: [check_target_prototypes_bin, '--relation', relation]
        + target_native_common_args + [
        '--report', '@OUTPUT1@',
        '--stamp', '@OUTPUT0@',
        '--executor-diagnostics-directory',
        meson.current_build_dir() / ('target-elements-prototype-' + relation
          + '-diagnostics'),
      ],
      build_always_stale: true,
      build_by_default: false,
      console: true,
    )
  endforeach

  alias_target('target-elements-prototype-check', target_prototype_checks)
endif
