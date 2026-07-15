project('tripod', version: '0.1.0-dev', subproject_dir: 'papers')

# ---------------------------------------------------------------------------
# External tools required for the LaTeX build.
# ---------------------------------------------------------------------------
latexmk = find_program('latexmk', required: true)
xelatex = find_program('xelatex', required: true)
biber = find_program('biber', required: true)

# ---------------------------------------------------------------------------
# Rust toolchain. Configuration never compiles the workspace: the paper
# subproject declares one custom target per required helper binary
# (execwrap, flatten-latex-main), each running
# scripts/cargo-bin-sync.sh — `cargo build --release --locked` for that
# binary plus a copy-if-changed into the declared output — so the paper
# targets carry real dependency edges and cargo remains the Rust
# dependency tracker.
# ---------------------------------------------------------------------------
cargo = find_program('cargo', required: true)
cargo_manifest = meson.project_source_root() / 'Cargo.toml'

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
# Paper subproject.
# ---------------------------------------------------------------------------
attestation_proj = subproject('attestation', default_options: latex_default_options)

attestation_dep = attestation_proj.get_variable('document_dep')

# `install_pdf` is the canonical build-dir artifact consumed by `meson
# install`. The mirror target copies that PDF into <repo>/archive/rendered/,
# and the flatten target emits the single-file .tex into
# <repo>/archive/flattened/. The `attestation` alias bundles both so
# `meson compile -C build attestation` exports both deliverables.
attestation_pdf = attestation_proj.get_variable('install_pdf')
attestation_mirror = attestation_proj.get_variable('mirror_pdf_target')
attestation_flatten = attestation_proj.get_variable('flatten_target')

alias_target('attestation', attestation_mirror, attestation_flatten)
alias_target('flatten-attestation', attestation_flatten)
alias_target('papers', attestation_mirror, attestation_flatten)

# ---------------------------------------------------------------------------
# Rust lint / test / generated-artifact suites surfaced through meson test.
# Cargo always runs --locked: CI and document builds must not silently
# update dependency resolution.
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

# Non-writing stale-artifact gate: committed generated artifacts must
# equal their typed-source derivations (ADR-010 binary; JSON report on
# stdout, which the test harness captures).
test(
  'check-generated',
  cargo,
  args: [
    'run',
    '--manifest-path', cargo_manifest,
    '--package', 'tripod-artifacts',
    '--bin', 'check-generated',
  ],
  suite: 'generated',
  timeout: 600,
)
