#!/bin/sh
set -eu

stamp=''
if [ "${1:-}" = "--stamp" ]; then
	if [ "$#" -lt 2 ]; then
		echo "usage: scripts/check-forbidden-text.sh [--stamp PATH]" >&2
		exit 2
	fi
	stamp=$2
	shift 2
fi

if [ "$#" -ne 0 ]; then
	echo "usage: scripts/check-forbidden-text.sh [--stamp PATH]" >&2
	exit 2
fi

if [ -n "$stamp" ]; then
	case "$stamp" in
		/*) ;;
		*) stamp=$PWD/$stamp ;;
	esac
fi

cd "$(dirname "$0")/.."

needle=$(printf '<%s>' char)
set +e
matches=$(git grep -n -F "$needle" -- .)
grep_status=$?
set -e

case "$grep_status" in
	0)
		printf '%s\n' "$matches" >&2
		printf 'error: forbidden literal %s found in tracked files\n' "$needle" >&2
		printf 'hint: use Vec<_>, collect::<Vec<_>>(), or escaped prose spelling.\n' >&2
		exit 1
		;;
	1)
		;;
	*)
		echo "error: git grep failed while checking forbidden text" >&2
		exit "$grep_status"
		;;
esac

if [ -n "$stamp" ]; then
	: > "$stamp"
fi

echo "==> forbidden text check passed" >&2