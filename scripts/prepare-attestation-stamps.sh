#!/bin/sh
# Build-system internal wrapper (ADR-014): derive the four paper stamps
# with tripod-document-stamps, validate the JSON, and render
# the generated stamps.tex plus the source-date-epoch file. Fail-closed:
# any missing placeholder, leftover token, or malformed value aborts and
# leaves the declared outputs untouched. No persistent JSON is retained.
#
# Usage:
#   prepare-attestation-stamps.sh \
#     <program> <jq> <git> <repository-root> <tree-ref> <tree-path> \
#     <template> <tex-output> <epoch-output> <ok-output> [--input <path>]...
set -eu

program="$1"
jq_program="$2"
git_program="$3"
repository_root="$4"
tree_ref="$5"
tree_path="$6"
template="$7"
tex_output="$8"
epoch_output="$9"
shift 9
ok_output="$1"
shift

json_tmp="$(mktemp "${TMPDIR:-/tmp}/attestation-stamps-json-XXXXXX")"
tex_tmp="$(mktemp "$(dirname "$tex_output")/.stamps.tex.XXXXXX")"
epoch_tmp="$(mktemp "$(dirname "$epoch_output")/.source-date-epoch.XXXXXX")"

cleanup() {
    rm -f "$json_tmp" "$tex_tmp" "$epoch_tmp"
}
trap cleanup EXIT HUP INT TERM

# Derive the four values. The program refuses a dirty paper subtree and
# performs no writes; its sole output is one JSON object on stdout.
"$program" \
    --git "$git_program" \
    --repository-root "$repository_root" \
    --tree-ref "$tree_ref" \
    --tree "$tree_path" \
    "$@" \
    > "$json_tmp"

# Validate the JSON shape before trusting any field.
"$jq_program" -e '
    type == "object"
    and (keys == [
        "date",
        "document_uuid",
        "instance_uuid",
        "timestamp"
    ])
    and (.date | type == "string")
    and (.document_uuid | type == "string")
    and (.instance_uuid | type == "string")
    and (.timestamp | type == "object")
    and (.timestamp.epoch | type == "number")
    and (.timestamp.iso_8601 | type == "string")
    and (.timestamp.pdf | type == "string")
' "$json_tmp" > /dev/null

# Every placeholder must occur exactly once in the template, or the
# substitution below could silently drop a value.
for token in \
    ATTESTATION_DATE \
    ATTESTATION_TIMESTAMP \
    ATTESTATION_DOCUMENT_UUID \
    ATTESTATION_INSTANCE_UUID
do
    count="$(
        "$jq_program" -rn \
            --rawfile template "$template" \
            --arg token "@${token}@" \
            '[$template | scan($token)] | length'
    )"
    if [ "$count" != 1 ]; then
        echo "template placeholder @${token}@ occurs $count times, expected 1" >&2
        exit 1
    fi
done

# Render the template. Substitution order is fixed and each placeholder
# is distinct, so a value can never contain another placeholder. `-n`
# (null input) is required: the data arrives via --rawfile/--slurpfile,
# so without it jq would block reading stdin.
"$jq_program" -jnr \
    --rawfile template "$template" \
    --slurpfile values "$json_tmp" '
        $values[0] as $v
        | $template
        | gsub("@ATTESTATION_DATE@"; $v.date)
        | gsub("@ATTESTATION_TIMESTAMP@"; $v.timestamp.pdf)
        | gsub("@ATTESTATION_DOCUMENT_UUID@"; $v.document_uuid)
        | gsub("@ATTESTATION_INSTANCE_UUID@"; $v.instance_uuid)
    ' > "$tex_tmp"

# Fail-closed: no unresolved placeholder may survive.
if grep -q '@ATTESTATION_' "$tex_tmp"; then
    echo "generated stamps.tex still contains an unresolved placeholder" >&2
    exit 1
fi

# The epoch file carries only the integer, one line.
"$jq_program" -jr '(.timestamp.epoch | tostring), "\n"' "$json_tmp" > "$epoch_tmp"

case "$(cat "$epoch_tmp")" in
    '' | *[!0-9]*)
        echo "derived source epoch is not a positive integer" >&2
        exit 1
        ;;
esac

# Atomic, compare-if-changed replacement so ninja restat can skip
# downstream rebuilds when the derived values are unchanged.
replace_if_changed() {
    staged="$1"
    destination="$2"
    if test -f "$destination" && cmp -s "$staged" "$destination"; then
        rm -f "$staged"
    else
        mv "$staged" "$destination"
    fi
}

replace_if_changed "$tex_tmp" "$tex_output"
replace_if_changed "$epoch_tmp" "$epoch_output"

: > "$ok_output"
