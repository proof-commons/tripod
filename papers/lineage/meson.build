# Keyed Provenance Covenant specification and publication inputs.
project('lineage', version: '1.0.0', meson_version: '>=1.3.0')
fs = import('fs')
main_tex = files('main.tex')
latexmk_rc = files('.latexmkrc')
bib_file = files('references.bib')
macros_lineage_src = files('macros_lineage.tex')
stamps_template = files('stamps.tex.in')
lineage_section_files = files(
  'sections/00_title.tex',
  'sections/01_interface.tex',
  'sections/02_model.tex',
  'sections/03_operations.tex',
  'sections/04_invariants.tex',
  'sections/10_limitations.tex',
  'sections/A1_verification.tex',
  'sections/B1_symbols.tex',
)
lineage_excluded_files = macros_lineage_src + stamps_template
all_inputs = main_tex + latexmk_rc + bib_file + macros_lineage_src \
  + stamps_template + lineage_section_files
repository_root = meson.global_source_root()
cargo = find_program('cargo')
git = find_program(get_option('git_path'))
cargo_bin_sync = find_program(repository_root / 'scripts' / 'cargo-bin-sync.sh')
cargo_manifest = repository_root / 'Cargo.toml'
stamp_bin = custom_target(
  'cargo-lineage-stamps',
  output: 'attestation-stamps',
  command: [cargo_bin_sync, cargo, cargo_manifest,
            'tripod-document-stamps', 'attestation-stamps', '@OUTPUT@'],
  build_always_stale: true,
  console: true,
)
all_input_args = []
foreach input_file : all_inputs
  all_input_args += ['--input', fs.relative_to(input_file, repository_root)]
endforeach
lineage_stamps = custom_target(
  'generate-lineage-stamps',
  input: all_inputs,
  output: ['stamps.tex', 'source-date-epoch'],
  command: [stamp_bin, '--git', git.full_path(),
            '--repository-root', repository_root,
            '--tree-ref', 'HEAD', '--tree', 'papers/lineage',
            '--template', stamps_template,
            '--stamps-output', '@OUTPUT0@', '--epoch-output', '@OUTPUT1@']
           + all_input_args,
  build_always_stale: true,
  console: true,
)
if get_option('mock_mode')
  mock_tex = custom_target(
    'cargo-lineage-mock-tex',
    output: 'execwrap-mock-tex',
    command: [cargo_bin_sync, cargo, cargo_manifest,
              'execwrap', 'execwrap-mock-tex', '@OUTPUT@'],
    build_always_stale: true,
    console: true,
  )
  render_command = [mock_tex, '--child', 'latexmk', '--outdir', '@OUTDIR@']
  if get_option('mock_fail_child') == 'latexmk'
    render_command += ['--fail']
  endif
else
  xelatex = find_program(get_option('xelatex_path'))
  latexmk = find_program(get_option('latexmk_path'))
  biber = find_program(get_option('biber_path'))
  execwrap = custom_target(
    'cargo-lineage-execwrap',
    output: 'execwrap',
    command: [cargo_bin_sync, cargo, cargo_manifest,
              'execwrap', 'execwrap', '@OUTPUT@'],
    build_always_stale: true,
    console: true,
  )
  epoch_wrap = find_program(repository_root / 'scripts' / 'with-source-date-epoch.sh')
  # The transcript includes preliminary passes; main.log is the converged TeX log.
  render_command = [execwrap, '--quiet', '--notify-on-write', '--redirect',
    meson.current_build_dir() / 'render-transcript.txt', '--',
    epoch_wrap, meson.current_build_dir() / 'source-date-epoch', latexmk,
    '-r', latexmk_rc[0], '-output-directory=@OUTDIR@',
    '-halt-on-error', '-interaction=nonstopmode',
    '-bibtex-', '-quiet', '-silent', '-time', main_tex[0]]
endif
copy_references = custom_target(
  'lineage-copy-references',
  input: bib_file,
  output: 'references.bib',
  command: ['cp', '@INPUT@', '@OUTPUT@'],
)
if get_option('mock_mode')
  pre_command = [mock_tex, '--child', 'xelatex', '--outdir', '@OUTDIR@']
  bib_command = [mock_tex, '--child', 'biber', '--outdir', '@OUTDIR@']
else
  # Preliminary passes retain diagnostics separately from the converged TeX log.
  pre_command = [execwrap, '--quiet', '--notify-on-write', '--redirect',
    meson.current_build_dir() / 'pre-generate-transcript.txt', '--',
    epoch_wrap, meson.current_build_dir() / 'source-date-epoch', xelatex,
    '-halt-on-error', '-interaction=nonstopmode',
    '-output-directory=@OUTDIR@', '-no-pdf', main_tex[0]]
  bib_command = [execwrap, '--quiet', '--change-directory',
    meson.current_build_dir(), '--notify-on-write', '--redirect',
    meson.current_build_dir() / 'bibliography-transcript.txt', '--',
    epoch_wrap, meson.current_build_dir() / 'source-date-epoch', biber, 'main']
endif
pre_generate = custom_target(
  'lineage-pre-generate',
  input: all_inputs + [lineage_stamps[0], lineage_stamps[1]],
  output: 'main.bcf',
  command: pre_command,
  depends: [copy_references, lineage_stamps],
  env: {'TEXINPUTS': meson.current_build_dir() + ':' + meson.current_source_dir() + ':'},
  console: true,
)
generate_bibliography = custom_target(
  'lineage-generate-bibliography',
  input: [pre_generate, lineage_stamps[1]],
  output: 'main.bbl',
  command: bib_command,
  depends: [copy_references, lineage_stamps],
  env: {'TEXINPUTS': meson.current_build_dir() + ':' + meson.current_source_dir() + ':'},
  console: true,
)
render_document = custom_target(
  'render-lineage-document',
  input: all_inputs + [lineage_stamps[0], lineage_stamps[1]],
  output: 'main.pdf',
  command: render_command,
  env: {'TEXINPUTS': meson.current_build_dir() + ':' + meson.current_source_dir() + ':'},
  depends: [generate_bibliography, lineage_stamps],
  console: true,
)
install_pdf = custom_target(
  'lineage-named-pdf',
  input: render_document,
  output: 'Tripod_Keyed_Provenance_Covenant_v@0@.pdf'.format(meson.project_version()),
  command: ['cp', '@INPUT@', '@OUTPUT@'],
  install: true,
  install_dir: join_paths(meson.global_source_root(), 'archive', 'rendered'),
  install_tag: 'doc',
  console: false,
)

flatten_latex_main = custom_target(
  'cargo-lineage-flatten-latex-main',
  output: 'flatten-latex-main',
  command: [
    cargo_bin_sync, cargo, cargo_manifest,
    'flatten-latex-main', 'flatten-latex-main', '@OUTPUT@',
  ],
  console: true,
  build_always_stale: true,
)

flat_basename = 'Tripod_Keyed_Provenance_Covenant_v@0@_flat.tex'.format(meson.project_version())

# The flattener is directory-blind (ADR-015): it never looks a file up by
# directory. It inlines only the files on this explicit list, matching each
# \input / \subfile / \addbibresource reference by its trailing path
# components. A reference to anything not listed — an absolute path, a `..`
# escape, or a symlink target off the list — is refused rather than read into
# the publication. The generated stamps.tex is the build-dir member; the
# sections, macros, and bibliography are the source members.
flatten_allowed = lineage_section_files + macros_lineage_src + bib_file \
  + [lineage_stamps[0]]
flatten_file_args = []
foreach allowed_file : flatten_allowed
  flatten_file_args += ['--file', allowed_file]
endforeach

flat_build_target = custom_target(
  'flatten-lineage-build',
  input: all_inputs + [lineage_stamps[0]],
  output: flat_basename,
  command: [
    flatten_latex_main,
    '--main', main_tex[0],
    '--output', '@OUTPUT@',
    '--strict-bib',
  ] + flatten_file_args,
  depends: lineage_stamps,
  console: false,
)
