project('tripod', version: '0.1.0-dev', subproject_dir: 'papers')

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

# ---------------------------------------------------------------------------
# Reproducible document builds (ADR-011): TeX date material derives from
# the source commit epoch, not the ambient clock, so identical inputs
# render byte-identical PDFs. Falls back to the Unix epoch outside a
# git checkout. Note: the value is captured at configure time; rerun
# meson setup (or configure) after committing to refresh it.
# ---------------------------------------------------------------------------
git = find_program('git', required: false)
source_date_epoch = '0'
zero_id = '0000000000000000000000000000000000000000'
source_commit = zero_id
source_tree = zero_id
if git.found()
  epoch_result = run_command(
    git, '-C', meson.project_source_root(), 'log', '-1', '--format=%ct',
    check: false,
  )
  if epoch_result.returncode() == 0 and epoch_result.stdout().strip() != ''
    source_date_epoch = epoch_result.stdout().strip()
  endif
  commit_result = run_command(
    git, '-C', meson.project_source_root(), 'log', '-1', '--format=%H',
    check: false,
  )
  if commit_result.returncode() == 0 and commit_result.stdout().strip() != ''
    source_commit = commit_result.stdout().strip()
  endif
  tree_result = run_command(
    git, '-C', meson.project_source_root(), 'log', '-1', '--format=%T',
    check: false,
  )
  if tree_result.returncode() == 0 and tree_result.stdout().strip() != ''
    source_tree = tree_result.stdout().strip()
  endif
endif

latex_default_options = [
  'source_date_epoch=' + source_date_epoch,
  'source_commit=' + source_commit,
  'source_tree=' + source_tree,
  'xelatex_path=' + xelatex.full_path(),
  'biber_path=' + biber.full_path(),
  'latexmk_path=' + latexmk.full_path(),
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
realization_file = attestation_proj.get_variable('realization_doc_file')

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
subdir('plans')
subdir('packages')

adr_files = adr_record_files

# Repository-root and subproject documentation (DOC owner).
doc_files = files('AGENTS.md', 'README.md') \
  + adr_doc_files + packages_doc_files \
  + attestation_proj.get_variable('human_doc_file')

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

# Role-tagged argument lists. Paths stay repository-relative: every
# binary resolves them against --repository-root, while the file
# objects above give ninja its dependency edges.
scoped_census_args = [
  '--repository-root', meson.project_source_root(),
  '--attestation-main', attestation_main_path,
  '--realization', realization_path,
]
foreach p : attestation_section_paths
  scoped_census_args += ['--attestation-section', p]
endforeach
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

# Census audit: build_always_stale re-runs the diff on every ninja
# invocation; the stamp is created once and never re-dated on a no-op
# run, so restat keeps the lint targets below clean.
census_audit = custom_target(
  'census-audit',
  output: 'census.ok',
  command: [
    find_program('scripts/census-audit.sh'),
    meson.project_source_root(),
    '@OUTPUT@',
  ] + census_paths,
  build_always_stale: true,
  build_by_default: true,
  console: false,
)

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
# slice plus the checker binary, the output is a build-dir stamp the
# checker touches on success. A rerun on an unchanged tree executes
# nothing.
# ---------------------------------------------------------------------------
scoped_census_inputs = attestation_main_file + attestation_section_files + realization_file \
                       + model_source_files

labels_stamp = custom_target(
  'labels-check',
  input: scoped_census_inputs + adr_files + plan_files + doc_files
         + crate_source_files + specification_register_file + realization_register_file
         + model_labels_json_file,
  output: 'labels.ok',
  command: [check_labels_bin] + labels_census_args + ['--stamp', '@OUTPUT@'],
  depends: census_audit,
  build_by_default: true,
  console: true,
)

generated_stamp = custom_target(
  'generated-check',
  input: scoped_census_inputs + generated_files + model_labels_json_file,
  output: 'generated.ok',
  command: [check_generated_bin] + scoped_census_args + [
    '--generated-dir', generated_dir,
    '--stamp', '@OUTPUT@',
  ],
  depends: census_audit,
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
  depends: census_audit,
  build_by_default: true,
  console: true,
)

alias_target('lint', labels_stamp, generated_stamp, plans_stamp)

# ---------------------------------------------------------------------------
# Generators: committed in-tree publications cannot be declared
# outputs, so a shell wrapper touches the build-dir stamp after the
# generator succeeds (ADR-014 generator-stamp rule). On-demand targets;
# regeneration is incremental over the same census slices.
# ---------------------------------------------------------------------------
generator_wrap = [sh, '-c', 'stamp="$1"; shift; "$@" && : > "$stamp"', 'sh', '@OUTPUT@']

generate_registers_target = custom_target(
  'run-generate-label-registers',
  input: scoped_census_inputs,
  output: 'generate-label-registers.ok',
  command: generator_wrap + [generate_registers_bin] + scoped_census_args + [
    '--specification-register-output', specification_register_path,
    '--realization-register-output', realization_register_path,
  ],
  build_by_default: false,
  console: true,
)
alias_target('generate-label-registers', generate_registers_target)

generate_artifacts_target = custom_target(
  'run-generate-artifacts',
  input: scoped_census_inputs,
  output: 'generate-artifacts.ok',
  command: generator_wrap + [generate_all_bin] + scoped_census_args + [
    '--output', generated_dir,
  ],
  build_by_default: false,
  console: true,
)
alias_target('generate-artifacts', generate_artifacts_target)

# ---------------------------------------------------------------------------
# Archive exports. `install_pdf` is the canonical build-dir artifact
# consumed by `meson install`; the mirror targets copy the PDF into
# <repo>/archive/rendered/ and the flattened .tex into
# <repo>/archive/flattened/, gated on the label-check stamp (ADR-014):
# a publication must not embed an unlinted label graph. The plain
# in-build PDF compile stays independent for fast draft iteration.
# ---------------------------------------------------------------------------
mirror_wrap = [
  sh, '-c',
  'mkdir -p "$(dirname "$2")" && cp "$1" "$2" && : > "$3"',
  'sh', '@INPUT@',
]

attestation_mirror = custom_target(
  'mirror-attestation-pdf',
  input: attestation_pdf,
  output: 'attestation-pdf.mirrored.stamp',
  command: mirror_wrap + [
    meson.project_source_root() / 'archive' / 'rendered'
    / fs.name(attestation_pdf.full_path()),
    '@OUTPUT@',
  ],
  depends: labels_stamp,
  build_by_default: true,
  console: false,
)

attestation_flatten = custom_target(
  'flatten-attestation-mirror',
  input: attestation_flat,
  output: 'attestation-flat.mirrored.stamp',
  command: mirror_wrap + [
    meson.project_source_root() / 'archive' / 'flattened'
    / fs.name(attestation_flat.full_path()),
    '@OUTPUT@',
  ],
  depends: labels_stamp,
  build_by_default: true,
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

# clippy and the test suite compile the whole workspace on a cold cache, so
# they need far more than meson's default 30-second test timeout.
test(
  'cargo-clippy',
  cargo,
  args: [
    'clippy',
    '--workspace',
    '--all-targets',
    '--manifest-path', cargo_manifest,
    '--',
    '-D', 'warnings',
  ],
  suite: 'lint',
  timeout: 1800,
)

test(
  'cargo-test',
  cargo,
  args: ['test', '--workspace', '--locked', '--manifest-path', cargo_manifest],
  suite: 'unit',
  timeout: 1800,
)

stamp_probe = ['-c', 'test -f "$1"', 'sh']

test(
  'census-audit',
  sh,
  args: stamp_probe + [census_audit.full_path()],
  depends: census_audit,
  suite: 'lint',
  timeout: 60,
)

test(
  'check-generated',
  sh,
  args: stamp_probe + [generated_stamp.full_path()],
  depends: generated_stamp,
  suite: 'generated',
  timeout: 60,
)

test(
  'labels-check',
  sh,
  args: stamp_probe + [labels_stamp.full_path()],
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
