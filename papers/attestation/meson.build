# papers/attestation/meson.build - Attestation paper subproject

project('attestation', 'c', version: '1.0.0', meson_version: '>=1.3.0')

fs = import('fs')

# Paper configuration
main_tex = files('main.tex')
latexmk_rc = files('.latexmkrc')
bib_file = files('references.bib')

# Specification macros published for importing documents. Sibling
# subprojects stage a build-dir copy of this via configure_file instead of an
# in-tree symlink, so meson tracks edits across subproject boundaries.
macros_attestation_src = files('macros_attestation.tex')

# Source template for the generated stamps.tex (Git-derived provenance).
stamps_template = files('stamps.tex.in')

# Specification lint census (ADR-014): explicit and hand-managed — a new
# section must be listed here, or the census audit fails the build.
# The parent project imports these file objects for the label-check
# lane (meson's subproject sandbox forbids it grabbing the files
# itself) and derives repository-relative paths with fs.relative_to.
attestation_section_files = files(
  'sections/00_title.tex',
  'sections/01_interface.tex',
  'sections/02_model.tex',
  'sections/03_operations.tex',
  'sections/04_invariants.tex',
  'sections/05_costliness.tex',
  'sections/06_containment.tex',
  'sections/07_discipline.tex',
  'sections/08_seigniorage.tex',
  'sections/09_maturity.tex',
  'sections/10_limitations.tex',
  'sections/A1_verification.tex',
  'sections/B1_symbols.tex',
)

# Same-typed files deliberately outside the specification lint census
# (ADR-014): the macro file and the stamps template are TeX build
# inputs, not label subjects. The census audit accepts the exclusion
# only because both are declared here.
attestation_excluded_files = macros_attestation_src + stamps_template

# TeX dependencies (label subjects excluded from all_inputs are handled
# separately). The stamps template is a publication input but not a
# label source, so it enters all_inputs directly, not tex_dependencies.
tex_dependencies = macros_attestation_src + attestation_section_files

# The exact publication-input set fed to the stamp digest. The generated
# stamps.tex is deliberately NOT a member (it is derived from these), so
# the document identity has no self-reference.
all_inputs = main_tex + tex_dependencies + latexmk_rc + bib_file + stamps_template

# Tool paths come from parent project options. In mock_mode the TeX
# toolchain is simulated by execwrap --mock-child, so its programs are not
# discovered (the parent does not pass their paths).
mock_mode = get_option('mock_mode')
git = find_program(get_option('git_path'), required: true)
sh = find_program('sh')
if not mock_mode
  xelatex = find_program(get_option('xelatex_path'))
  biber = find_program(get_option('biber_path'))
  latexmk = find_program(get_option('latexmk_path'))
endif

# The generated stamps.tex lives in the build directory, so it must be
# searchable from there in addition to the source tree. SOURCE_DATE_EPOCH
# and FORCE_SOURCE_DATE are injected per-command by with-source-date-epoch.sh
# from the build-time generated epoch, not the ambient clock (ADR-011).
source_date_epoch_wrap = find_program(
  meson.global_source_root() / 'scripts' / 'with-source-date-epoch.sh'
)
reproducible_tex_env = {
  'TEXINPUTS':
    meson.current_build_dir() + ':' + meson.current_source_dir() + ':',
}

# Absolute path to the generated epoch file, for the with-source-date-epoch.sh
# argument. It must be absolute because the Biber step runs with a changed
# working directory; the dependency edge on the generating target is carried
# by each consumer's input/depends list, not this string.
source_date_epoch_path = meson.current_build_dir() / 'source-date-epoch'

# The two Rust helper binaries are real build targets, not
# configure-time discoveries: each custom target runs the repo's
# cargo-bin-sync script (`cargo build --release` for that one
# binary, then copy-if-changed into the declared output). Using the
# targets as the command of the paper steps below gives ninja genuine
# dependency edges — deleting a binary rebuilds it, and meson setup
# never compiles the workspace. cargo tracks the Rust sources itself
# (hence build_always_stale); the copy-if-changed plus ninja's restat
# keeps a no-op rebuild from re-rendering the paper.
cargo = find_program('cargo', required: true)
cargo_manifest = meson.global_source_root() / 'Cargo.toml'
cargo_bin_sync = find_program(meson.global_source_root() / 'scripts' / 'cargo-bin-sync.sh')

execwrap = custom_target(
  'cargo-execwrap',
  output: 'execwrap',
  command: [cargo_bin_sync, cargo, cargo_manifest, 'execwrap', 'execwrap', '@OUTPUT@'],
  console: true,
  build_always_stale: true,
)

flatten_latex_main = custom_target(
  'cargo-flatten-latex-main',
  output: 'flatten-latex-main',
  command: [
    cargo_bin_sync, cargo, cargo_manifest,
    'flatten-latex-main', 'flatten-latex-main', '@OUTPUT@',
  ],
  console: true,
  build_always_stale: true,
)

attestation_stamps_bin = custom_target(
  'cargo-attestation-stamps',
  output: 'attestation-stamps',
  command: [
    cargo_bin_sync, cargo, cargo_manifest,
    'tripod-document-stamps', 'attestation-stamps', '@OUTPUT@',
  ],
  console: true,
  build_always_stale: true,
)

# Build-time provenance stamps (ADR-014). The Rust binary derives four
# values from committed Git state and renders them directly into
# stamps.tex and source-date-epoch (and touches a .ok success probe):
# rendering, the fail-closed placeholder check, and the compare-if-changed
# writes all live in the binary and are unit-tested there. build_always_stale
# is required because ninja cannot otherwise see a changed HEAD, a paper-tree
# commit touching no all_inputs file, or a dirty working tree; the
# compare-if-changed writes let ninja restat skip downstream rebuilds when
# the derived values are unchanged.
repository_root = meson.global_source_root()
all_input_args = []
foreach input_file : all_inputs
  all_input_args += ['--input', fs.relative_to(input_file, repository_root)]
endforeach

attestation_stamps = custom_target(
  'generate-attestation-stamps',
  input: all_inputs,
  output: ['stamps.tex', 'source-date-epoch', 'attestation-stamps.ok'],
  command: [
    attestation_stamps_bin,
    '--git', git.full_path(),
    '--repository-root', repository_root,
    '--tree-ref', 'HEAD',
    '--tree', 'papers/attestation',
    '--template', stamps_template,
    '--stamps-output', '@OUTPUT0@',
    '--epoch-output', '@OUTPUT1@',
    '--stamp', '@OUTPUT2@',
  ] + all_input_args,
  build_always_stale: true,
  console: true,
)

# Success probe for the stamps generation (ADR-014 output-or-stamp).
test(
  'attestation-stamps',
  sh,
  args: ['-c', 'test -f "$1"', 'sh', attestation_stamps[2].full_path()],
  depends: attestation_stamps,
  suite: 'documentation',
  timeout: 60,
)

# The specification cites nothing outside itself: it imports no sibling .aux and stages no
# xref bridge. It only PUBLISHES its own .aux (below) for importing layers.

# Target 0: Copy bibliography file
copy_references = custom_target(
  'copy-references',
  input: bib_file,
  output: 'references.bib',
  command: ['cp', '@INPUT@', '@OUTPUT@'],
  console: false,
)

# Target 1: Generate .aux file using xelatex
pre_generate_log = meson.current_build_dir() / 'pre-generate.log'
if mock_mode
  pre_generate_command = [
    execwrap,
    '--quiet',
    '--notify-on-write',
    '--redirect', pre_generate_log,
    '--mock-child', 'xelatex',
    '--mock-outdir', '@OUTDIR@',
    '--', 'true',
  ]
else
  pre_generate_command = [
    execwrap,
    '--quiet',
    '--notify-on-write',
    '--redirect', pre_generate_log,
    '--', source_date_epoch_wrap, source_date_epoch_path, xelatex,
    '-halt-on-error',
    '-interaction=nonstopmode',
    '-shell-escape',
    '-output-directory=@OUTDIR@',
    '-no-pdf', main_tex[0],
  ]
endif
pre_generate = custom_target(
  'pre-generate',
  input: all_inputs + [attestation_stamps[0], attestation_stamps[1]],
  output: 'main.bcf',
  command: pre_generate_command,
  depends: [copy_references, attestation_stamps],
  env: reproducible_tex_env,
  console: true,
)

# Target 2: Run Biber
generate_bibliography_log = meson.current_build_dir() / 'generate-bibliography.log'
if mock_mode
  generate_bibliography_command = [
    execwrap,
    '--quiet',
    '--notify-on-write',
    '--redirect', generate_bibliography_log,
    '--mock-child', 'biber',
    '--mock-outdir', '@OUTDIR@',
    '--', 'true',
  ]
else
  generate_bibliography_command = [
    execwrap,
    '--quiet',
    '--change-directory', meson.current_build_dir(),
    '--notify-on-write',
    '--redirect', generate_bibliography_log,
    '--', source_date_epoch_wrap, source_date_epoch_path, biber,
    'main',
  ]
endif
generate_bibliography = custom_target(
  'generate-bibliography',
  input: [pre_generate, attestation_stamps[1]],
  output: 'main.bbl',
  command: generate_bibliography_command,
  depends: [copy_references, attestation_stamps],
  env: reproducible_tex_env,
  console: true,
)

# Target 3: Generate final PDF
render_document_log = meson.current_build_dir() / 'render-document.log'
if mock_mode
  render_mock_flags = ['--mock-child', 'latexmk', '--mock-outdir', '@OUTDIR@']
  if get_option('mock_fail_child') == 'latexmk'
    render_mock_flags += ['--mock-fail']
  endif
  render_document_command = [
    execwrap,
    '--quiet',
    '--notify-on-write',
    '--redirect', render_document_log,
  ] + render_mock_flags + ['--', 'true']
else
  render_document_command = [
    execwrap,
    '--quiet',
    '--notify-on-write',
    '--redirect', render_document_log,
    '--',
    source_date_epoch_wrap,
    source_date_epoch_path,
    latexmk,
    '-r', latexmk_rc[0],
    '-output-directory=@OUTDIR@',
    '-halt-on-error',
    '-interaction=nonstopmode',
    '-shell-escape',
    '-bibtex-',
    '-quiet',
    '-silent',
    '-time', main_tex[0],
  ]
endif
render_document = custom_target(
  'render-document',
  input: all_inputs + [attestation_stamps[0], attestation_stamps[1]],
  output: 'main.pdf',
  depends: [generate_bibliography, attestation_stamps],
  command: render_document_command,
  env: reproducible_tex_env,
  build_by_default: true,
  console: true,
)

# Target 4: Final named PDF — built into the build dir, installed via
# `meson install`, and ALSO mirrored into <repo>/archive/rendered/ on every
# build so the checked-in PDF tracks the latest sources (same pattern as
# the flatten target below).
install_pdf_name = 'Tripod_Attestation_v@0@.pdf'.format(meson.project_version())
install_pdf = custom_target(
  'install-pdf',
  input: render_document,
  output: install_pdf_name,
  command: ['cp', '@INPUT@', '@OUTPUT@'],
  install: true,
  install_dir: join_paths(meson.global_source_root(), 'archive', 'rendered'),
  install_tag: 'doc',
  console: false,
)

# The archive mirror lives in the top-level project (ADR-014): it
# gates on the label-check stamp, which is defined there.

# ── Cross-document reference bridge ────────────────────────────────────
# Republish the fully-converged .aux (written by latexmk alongside the PDF,
# so every label number is final) as a first-class, dependency-tracked
# artifact. Importing documents resolve specification labels through this via
# zref-xr (under the A- prefix) instead of reaching into our private build
# directory.
xref_aux = custom_target(
  'attestation-xref-aux',
  input: render_document,
  output: 'attestation.aux',
  command: ['cp', meson.current_build_dir() / 'main.aux', '@OUTPUT@'],
  console: false,
)

# Declare dependency for other projects to use
document_dep = declare_dependency(
  sources: [install_pdf],
)

# Flattened-LaTeX target. Inlines every `\input{}` into a single
# self-contained file. The .tex sources are listed as inputs so meson only
# re-runs the flattener when something actually changes. The build-dir copy
# is the canonical output for ninja's freshness tracking; the top-level
# project mirrors it into <repo>/archive/flattened/ (gated on the
# label-check stamp, ADR-014) so the checked-in copy stays in sync.
flat_basename = 'Tripod_Attestation_v@0@_flat.tex'.format(meson.project_version())
flat_build_target = custom_target(
  'flatten-attestation-build',
  input: all_inputs + [attestation_stamps[0]],
  output: flat_basename,
  command: [
    flatten_latex_main,
    '--paper-dir', meson.current_source_dir(),
    '--include-dir', meson.current_build_dir(),
    '--output', '@OUTPUT@',
    '--strict-bib',
  ],
  depends: attestation_stamps,
  console: false,
)
