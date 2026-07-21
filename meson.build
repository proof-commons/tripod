project(
  'tripod',
  version: '0.1.0-dev',
  meson_version: '>=1.3.0',
  subproject_dir: 'papers',
)

fs = import('fs')

# ---------------------------------------------------------------------------
# External tools required for the LaTeX build.
# ---------------------------------------------------------------------------
latexmk = find_program('latexmk', required: true)
xelatex = find_program('xelatex', required: true)
biber = find_program('biber', required: true)

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
python = find_program('python3')

# Git is used by the census audit and passed to document subprojects
# that derive build-time provenance metadata from the selected source
# revision. It is required for canonical publication.
git = find_program('git', required: true)

# jq validates and renders the derived stamp JSON into the generated
# stamps.tex and source-date-epoch (see scripts/prepare-attestation-stamps.sh).
jq = find_program('jq', required: true)

latex_default_options = [
  'xelatex_path=' + xelatex.full_path(),
  'biber_path=' + biber.full_path(),
  'latexmk_path=' + latexmk.full_path(),
  'git_path=' + git.full_path(),
  'jq_path=' + jq.full_path(),
]

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

adr_files = adr_record_files

# The realization document owns its own R13 labels; it lives under
# docs/attestation so its edits do not perturb the paper's identity.
realization_file = attestation_realization_file

# Repository-root and repository documentation (DOC owner).
doc_files = files('AGENTS.md', 'README.md') \
  + adr_doc_files + packages_doc_files \
  + docs_doc_files

# Repository-relative path strings for checker argv and the census
# audit, derived from the declared file objects.
source_root = meson.project_source_root()
attestation_main_path = fs.relative_to(attestation_main_file[0], source_root)
realization_path = fs.relative_to(realization_file[0], source_root)
specification_register_path = fs.relative_to(specification_register_file[0], source_root)
realization_register_path = fs.relative_to(realization_register_file[0], source_root)
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

census_paths = [attestation_main_path, realization_path, specification_register_path,
                realization_register_path, model_labels_json_path] \
               + attestation_section_paths + adr_paths + plan_paths + doc_paths \
               + model_source_paths + crate_source_paths + generated_paths

# Tracked paths that are categorically never lint subjects: dotfiles,
# archives, helper scripts, build and package definitions, licences,
# and binary or data blobs. Same-typed exceptions (a stray .tex or .rs
# that is deliberately not a subject) are NOT listed here — they live
# in their directory's explicit exclusion list.
census_exclude_pattern = '(^|/)[.]|^archive/|^scripts/|^LICENSE-' \
  + '|(^|/)meson[.](build|options)$|(^|/)Cargo[.]toml$|[.](bib|txt|zip)$'

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
# the build by itself. Checker stdout is ADR-010 JSON result data, so
# Meson declares it as a build output instead of exposing it to the
# interactive terminal inherited by console targets.
checker_wrap = [
  sh, '-c',
  'report="$1"; program="$2"; shift 2; case "$program" in */*) exec "$program" "$@" > "$report" ;; *) exec "./$program" "$@" > "$report" ;; esac',
  'sh',
]

census_audit = custom_target(
  'census-audit',
  output: ['census.ok', 'census-audit.json'],
  command: checker_wrap + ['@OUTPUT1@', census_audit_bin,
    '--repository-root', source_root,
    '--git', git.full_path(),
    '--exclude-pattern', census_exclude_pattern,
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
         + model_labels_json_file,
  output: ['labels.ok', 'labels-check.json'],
  command: checker_wrap + ['@OUTPUT1@', check_labels_bin] + labels_census_args + [
    '--stamp', '@OUTPUT0@',
  ],
  build_by_default: true,
  console: true,
)

generated_stamp = custom_target(
  'generated-check',
  input: scoped_census_inputs + generated_files + model_labels_json_file,
  output: ['generated.ok', 'generated-check.json'],
  command: checker_wrap + ['@OUTPUT1@', check_generated_bin] + scoped_census_args + [
    '--generated-dir', generated_dir,
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
plans_lane_files += specification_register_file + realization_register_file
plans_lane_paths += [specification_register_path, realization_register_path]

plans_stamp = custom_target(
  'plans-check',
  input: plans_lane_files + [files('scripts/check_plans.py')],
  output: 'plans.ok',
  command: [
    python, files('scripts/check_plans.py'),
    '--repository-root', meson.project_source_root(),
    '--stamp', '@OUTPUT@',
  ] + plans_lane_paths,
  build_by_default: true,
  console: true,
)

forbidden_text_script = files('scripts/check-forbidden-text.sh')
forbidden_text_stamp = custom_target(
  'forbidden-text-check',
  input: forbidden_text_script,
  output: 'forbidden-text.ok',
  command: [
    sh, forbidden_text_script,
    '--stamp', '@OUTPUT@',
  ],
  build_by_default: true,
  build_always_stale: true,
  console: true,
)

alias_target('lint', labels_stamp, generated_stamp, plans_stamp, forbidden_text_stamp)

# ---------------------------------------------------------------------------
# Generators: committed in-tree publications cannot be declared
# outputs, so a shell wrapper touches the build-dir stamp after the
# generator succeeds (ADR-014 generator-stamp rule). On-demand targets
# are always stale so an explicit repair request reruns the generator
# even when the stamp is newer than a corrupted committed publication.
# ---------------------------------------------------------------------------
generator_wrap = [
  sh, '-c',
  'stamp="$1"; program="$2"; shift 2; case "$program" in */*) "$program" "$@" ;; *) "./$program" "$@" ;; esac && : > "$stamp"',
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
# manifests, or Cargo's own dependency graph changed.
cargo_quiet_stamp_wrap = [
  sh, '-c',
  'stamp="$1"; log="$2"; shift 2; if CARGO_TERM_PROGRESS_WHEN=never "$@" > "$log" 2>&1; then : > "$stamp"; else cat "$log" >&2; exit 1; fi',
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

test(
  'cargo-clippy',
  sh,
  args: stamp_probe + [cargo_clippy_stamp[0].full_path()],
  depends: cargo_clippy_stamp,
  suite: 'lint',
  timeout: 60,
)

cargo_test_stamp = custom_target(
  'cargo-test-check',
  output: ['cargo-test.ok', 'cargo-test.log'],
  command: cargo_quiet_stamp_wrap + [
    cargo,
    'test',
    '--workspace',
    '--manifest-path', cargo_manifest,
  ],
  build_by_default: false,
  build_always_stale: true,
  console: true,
)

test(
  'cargo-test',
  sh,
  args: stamp_probe + [cargo_test_stamp[0].full_path()],
  depends: cargo_test_stamp,
  suite: 'unit',
  timeout: 60,
)

test(
  'census-audit',
  sh,
  args: stamp_probe + [census_audit[0].full_path()],
  depends: census_audit,
  suite: 'lint',
  timeout: 60,
)

test(
  'check-generated',
  sh,
  args: stamp_probe + [generated_stamp[0].full_path()],
  depends: generated_stamp,
  suite: 'generated',
  timeout: 60,
)

test(
  'labels-check',
  sh,
  args: stamp_probe + [labels_stamp[0].full_path()],
  depends: labels_stamp,
  suite: ['documentation', 'generated'],
  timeout: 60,
)

test(
  'plans-check',
  sh,
  args: stamp_probe + [plans_stamp.full_path()],
  depends: plans_stamp,
  suite: 'documentation',
  timeout: 60,
)

test(
  'forbidden-text-check',
  sh,
  args: stamp_probe + [forbidden_text_stamp.full_path()],
  depends: forbidden_text_stamp,
  suite: 'lint',
  timeout: 60,
)

generator_repair_test = find_program('scripts/test-meson-generator-repair.sh')
publication_repair_test = find_program('scripts/test-meson-publication-repair.sh')

test(
  'meson-generator-repair',
  generator_repair_test,
  args: [meson.project_source_root()],
  suite: 'build-integration',
  timeout: 600,
  is_parallel: false,
)

test(
  'meson-publication-repair',
  publication_repair_test,
  args: [meson.project_source_root()],
  suite: 'build-integration',
  timeout: 900,
  is_parallel: false,
)
