# papers/attestation/meson.build - Attestation paper subproject

project('attestation', 'c', version: '1.0.0')

# Paper configuration
main_tex = files('main.tex')
latexmk_rc = files('.latexmkrc')
bib_file = files('references.bib')

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
realization_doc_file = files('realization.md')
human_doc_file = files('human.md')

# TeX dependencies
tex_dependencies = files('macros_attestation.tex') + attestation_section_files

# Source template for the generated stamps.tex. It is a TeX build input
# rather than a label subject, and it is listed among the inputs so an
# edit to it re-renders the document.
stamps_template = files('stamps.tex.in')

all_inputs = main_tex + tex_dependencies + latexmk_rc + bib_file + stamps_template

# Specification macros published for importing documents. Sibling
# subprojects stage a build-dir copy of this via configure_file instead of an
# in-tree symlink, so meson tracks edits across subproject boundaries.
macros_attestation_src = files('macros_attestation.tex')

# TeX tool paths come from parent project options.
xelatex = find_program(get_option('xelatex_path'))
biber = find_program(get_option('biber_path'))
latexmk = find_program(get_option('latexmk_path'))

# The document's four provenance stamps derive from the same committed
# state the epoch does, so two builds of one source render identical
# bytes and no value follows the ambient clock: the visible date and the
# timestamp from the source epoch, the document identity from the commit
# that publishes the source, the instance identity from the tree that is
# rendered. `main.tex` inputs the generated file, and the flattener is
# given the build directory below so the self-contained source carries
# the macros rather than an unresolved input.
date_program = find_program('date', required: true)
stamp_date = run_command(
  date_program, '-u', '-d', '@' + get_option('source_date_epoch'), '+%Y-%m-%d',
  check: true,
).stdout().strip()
stamp_timestamp = run_command(
  date_program, '-u', '-d', '@' + get_option('source_date_epoch'), '+%Y-%m-%dT%H:%M:%SZ',
  check: true,
).stdout().strip()

stamp_commit = get_option('source_commit')
stamp_tree = get_option('source_tree')

stamps_configuration = configuration_data()
stamps_configuration.set('ATTESTATION_DATE', stamp_date)
stamps_configuration.set('ATTESTATION_TIMESTAMP', stamp_timestamp)
stamps_configuration.set(
  'ATTESTATION_DOCUMENT_UUID',
  '@0@-@1@-@2@-@3@-@4@'.format(
    stamp_commit.substring(0, 8), stamp_commit.substring(8, 12),
    stamp_commit.substring(12, 16), stamp_commit.substring(16, 20),
    stamp_commit.substring(20, 32),
  ),
)
stamps_configuration.set(
  'ATTESTATION_INSTANCE_UUID',
  '@0@-@1@-@2@-@3@-@4@'.format(
    stamp_tree.substring(0, 8), stamp_tree.substring(8, 12),
    stamp_tree.substring(12, 16), stamp_tree.substring(16, 20),
    stamp_tree.substring(20, 32),
  ),
)

stamps_tex = configure_file(
  input: stamps_template,
  output: 'stamps.tex',
  configuration: stamps_configuration,
)

# Reproducible TeX environment: \today, pdfdate, PDF CreationDate, and
# the trailer ID all derive from the pinned source epoch instead of the
# ambient clock (ADR-011). The build directory joins TEXINPUTS so
# xelatex resolves the generated stamps.
reproducible_tex_env = {
  'TEXINPUTS': meson.current_source_dir() + ':' + meson.current_build_dir() + ':',
  'SOURCE_DATE_EPOCH': get_option('source_date_epoch'),
  'FORCE_SOURCE_DATE': '1',
}

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
pre_generate = custom_target(
  'pre-generate',
  input: all_inputs,
  output: 'main.bcf',
  command: [
    execwrap,
    '--quiet',
    '--notify-on-write',
    '--redirect', pre_generate_log,
    '--', xelatex,
    '-halt-on-error',
    '-interaction=nonstopmode',
    '-shell-escape',
    '-output-directory=@OUTDIR@',
    '-no-pdf', main_tex[0],
  ],
  depends: [copy_references],
  env: reproducible_tex_env,
  console: true,
)

# Target 2: Run Biber
generate_bibliography_log = meson.current_build_dir() / 'generate-bibliography.log'
generate_bibliography = custom_target(
  'generate-bibliography',
  input: pre_generate,
  output: 'main.bbl',
  command: [
    execwrap,
    '--quiet',
    '--change-directory', meson.current_build_dir(),
    '--notify-on-write',
    '--redirect', generate_bibliography_log,
    '--', biber,
    'main',
  ],
  depends: copy_references,
  env: reproducible_tex_env,
  console: true,
)

# Target 3: Generate final PDF
render_document_log = meson.current_build_dir() / 'render-document.log'
render_document = custom_target(
  'render-document',
  input: all_inputs,
  output: 'main.pdf',
  depends: generate_bibliography,
  command: [
    execwrap,
    '--quiet',
    '--notify-on-write',
    '--redirect', render_document_log,
    '--',
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
  ],
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
  input: all_inputs,
  output: flat_basename,
  command: [
    flatten_latex_main,
    '--paper-dir', meson.current_source_dir(),
    '--include-dir', meson.current_build_dir(),
    '--output', '@OUTPUT@',
    '--strict-bib',
  ],
  console: false,
)
