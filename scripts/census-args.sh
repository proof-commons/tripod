# shellcheck shell=sh
#
# Shared, filename-safe derivation of role-tagged checker argv from the
# Git index (ADR-014). Sourced by scripts/check-plans.sh and by the
# tracked-path-argv-audit meson lane; it defines functions only and is
# not executable on its own.
#
# The checkers take their census by argument, one role tag plus one
# path per subject. Building that argv by unquoted command substitution
# was unsound (SR3-05): a tracked path containing whitespace splits
# into several arguments, a glob character can be expanded against the
# working directory, and a path beginning with a hyphen arrives where a
# checker option is expected. Git permits all three.
#
# Two independent rules replace the "paths here never contain
# whitespace" comment that used to stand in for a check:
#
#   1. `audit_tracked_paths` enforces the safe grammar, loudly. It is
#      an audit, never a silent normalization: an offending tracked
#      path fails the run and must be renamed. Paths carrying control
#      characters or non-ASCII bytes reach this loop in Git's C-quoted
#      form, which contains a double quote and so leaves the grammar
#      too.
#
#   2. Every derived argument is emitted shell-quoted and re-parsed by
#      the caller with `eval "set -- ..."`, so argv is assembled from
#      quoted words even if rule 1 were ever relaxed: no word
#      splitting, no globbing, and no path-as-option. Should a
#      hyphen-leading path still reach a checker, clap refuses it as a
#      missing option value rather than silently accepting it as one.
#
# Callers must treat an empty census as a failure: a checker invoked
# with no subjects would otherwise report a vacuous pass.

# Emit one shell-quoted word plus a separating space.
quote_word() {
  case "$1" in
    *"'"*)
      printf "'"
      printf '%s' "$1" | sed "s/'/'\\\\''/g"
      printf "' "
      ;;
    *) printf "'%s' " "$1" ;;
  esac
}

# Emit one role tag and its shell-quoted path.
emit_tagged() {
  printf '%s ' "$1"
  quote_word "$2"
}

# Refuse tracked paths that cannot be carried safely as checker argv.
audit_tracked_paths() {
  offending="$(
    git ls-files | while IFS= read -r path; do
      case "$path" in
        -* | *[!-A-Za-z0-9._/]*) printf '%s\n' "$path" ;;
      esac
    done
  )"

  if [ -n "$offending" ]; then
    printf '%s\n' "$offending" >&2
    echo "ERROR: the tracked paths above leave the checker-argv grammar" >&2
    echo "       (allowed: [-A-Za-z0-9._/], no leading hyphen); rename them" >&2
    return 1
  fi
}

# census_args <labels|scoped>: role-tagged checker argv, shell-quoted.
# The exclusion cases mirror the meson census: categorical non-subjects
# first, then the same-typed exclusions declared in per-directory
# meson.build lists (macros_attestation.tex, execwrap integration
# tests). Under scripts/ only the shell sources are categorical: the
# Python sources are label subjects (ADR-023), tagged below.
census_args() {
  git ls-files | LC_ALL=C sort | while IFS= read -r path; do
    case "$path" in
      .* | */.* | archive/* | scripts/*.sh) continue ;;
      papers/attestation/macros_attestation.tex) continue ;;
      papers/attestation/stamps.tex.in) continue ;;
      packages/execwrap/tests/*) continue ;;
      papers/attestation/main.tex) emit_tagged --attestation-main "$path" ;;
      papers/attestation/sections/*.tex) emit_tagged --attestation-section "$path" ;;
      docs/attestation/realization.md) emit_tagged --realization "$path" ;;
      packages/model/src/*.rs) emit_tagged --model-source "$path" ;;
      *)
        [ "$1" = labels ] || continue
        case "$path" in
          plans/labels/specification.md) emit_tagged --specification-register "$path" ;;
          plans/labels/realization.md) emit_tagged --realization-register "$path" ;;
          plans/labels/attestation.md) emit_tagged --attestation-register "$path" ;;
          packages/model/generated/model_labels.json) emit_tagged --model-labels-json "$path" ;;
          plans/*.md) emit_tagged --plan "$path" ;;
          adr/[0-9][0-9][0-9]*.md) emit_tagged --adr "$path" ;;
          packages/*/src/*.rs) emit_tagged --crate-source "$path" ;;
          scripts/*.py) emit_tagged --script-source "$path" ;;
          *.md) emit_tagged --doc "$path" ;;
        esac
        ;;
    esac
  done
}

# plan_subject_args: `--subject` argv for check-plans, shell-quoted.
plan_subject_args() {
  git ls-files adr plans | LC_ALL=C sort | while IFS= read -r path; do
    case "$path" in
      *.md) emit_tagged --subject "$path" ;;
    esac
  done
}

# derive_census <description> <command...>: run the census command and
# refuse an empty result. Callers must capture it in a plain assignment
# first, so `set -e` still sees the failure — a command substitution
# inside `eval` would swallow it:
#
#   census="$(derive_census 'label census' census_args labels)"
#   eval "set -- $census"
derive_census() {
  description="$1"
  shift

  derived="$("$@")"
  if [ -z "$derived" ]; then
    echo "ERROR: the $description is empty; refusing a vacuous checker run" >&2
    return 1
  fi

  printf '%s' "$derived"
}
