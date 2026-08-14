#!/bin/sh
# Zero-argument launcher for the target-native executor adapter.
#
# The conformance harness spawns the program named by --executor with no
# arguments and nulls its stderr (Guide-9 section 11.1, section 11.2), so the
# adapter's own configuration has to be established outside the first-party
# interface. This launcher is that boundary: it reads a small set of
# environment variables naming the node binaries and the upstream
# functional-test framework, and re-execs elements-native-executor.py with
# them as explicit arguments.
#
# None of these variables is a credential, and none may become one. Guide-9
# section 17.6 bans an RPC user, an RPC password, a cookie path, a token, a
# wallet, and a private key from this boundary; the node's cookie lives and
# dies inside the disposable data directory the adapter creates, and nothing
# here ever names it.
#
#   ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD      path to elementsd (required)
#   ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI   path to elements-cli (required)
#   ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK      upstream test_framework package,
#                                           or the directory holding it
#                                           (required)
#   ELEMENTS_NATIVE_EXECUTOR_UPSTREAM_REPO  upstream checkout, consulted for
#                                           a revision only when the binary
#                                           embeds none (optional)
#   ELEMENTS_NATIVE_EXECUTOR_PYTHON         python interpreter (default
#                                           python3)
#   ELEMENTS_NATIVE_EXECUTOR_LAUNCH_PREFIX  word-split command placed in
#                                           front of the interpreter, for
#                                           hosts where the node binaries
#                                           only run inside a container
#                                           (optional)
#
# Any further arguments given to this launcher are appended to the adapter's
# own, which is useful when running it by hand; the harness supplies none.
#
# Stdout is protocol data. Stderr is diagnostics the harness discards.
set -eu

here="$(cd "$(dirname "$0")" && pwd)"
adapter="$here/elements-native-executor.py"

missing=""
for name in ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD \
            ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI \
            ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK; do
  eval "value=\${$name:-}"
  if [ -z "$value" ]; then
    missing="$missing $name"
  fi
done
if [ -n "$missing" ]; then
  echo "elements-native-executor.sh: unset:$missing" >&2
  exit 1
fi

set -- --elementsd "$ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD" \
       --elements-cli "$ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI" \
       --framework "$ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK" \
       "$@"

if [ -n "${ELEMENTS_NATIVE_EXECUTOR_UPSTREAM_REPO:-}" ]; then
  set -- --upstream-repo "$ELEMENTS_NATIVE_EXECUTOR_UPSTREAM_REPO" "$@"
fi

python="${ELEMENTS_NATIVE_EXECUTOR_PYTHON:-python3}"

# Deliberately word-split: the prefix is a command, not one path.
# shellcheck disable=SC2086
exec ${ELEMENTS_NATIVE_EXECUTOR_LAUNCH_PREFIX:-} "$python" "$adapter" "$@"
