#!/bin/sh
# Render the role-tagged census (census.sh) into checker argv (ADR-014).
#
# Prints a whitespace-separated argument string for one consumer; paths
# never contain whitespace (census.sh rejects them), so callers expand
# the output unquoted:
#
#   cargo run ... --bin check-labels -- --repository-root . \
#     $(sh scripts/census-args.sh . labels)
#
# Modes:
#   labels   full census argv for check-labels
#   scoped   Layer-0 + realization + model argv for the scoped
#            derivations (generate-label-registers, check-generated,
#            generate-all)
#
# Usage: census-args.sh <repo-root> <labels|scoped>
set -eu

root="$1"
mode="$2"

sh "$(dirname "$0")/census.sh" "$root" | while IFS='	' read -r role path; do
  case "$role" in
    attestation-main) printf ' --attestation-main %s' "$path" ;;
    attestation-section) printf ' --attestation-section %s' "$path" ;;
    realization) printf ' --realization %s' "$path" ;;
    model-source) printf ' --model-source %s' "$path" ;;
    *)
      [ "$mode" = labels ] || continue
      case "$role" in
        adr) printf ' --adr %s' "$path" ;;
        plan) printf ' --plan %s' "$path" ;;
        doc) printf ' --doc %s' "$path" ;;
        crate-source) printf ' --crate-source %s' "$path" ;;
        specification-register) printf ' --specification-register %s' "$path" ;;
        realization-register) printf ' --realization-register %s' "$path" ;;
        model-labels-json) printf ' --model-labels-json %s' "$path" ;;
      esac
      ;;
  esac
done
printf '\n'
