#!/bin/sh
# Configuration launcher for the target-native executor adapter.
#
# The conformance harness spawns the program named by --executor with exactly
# two arguments -- --output and --elements-output, the two files the adapter
# writes its diagnostics into -- and with nothing else, so the adapter's own
# configuration has to be established outside the first-party interface. This
# launcher is that boundary: it reads a small set of environment variables
# naming the node binaries and the upstream functional-test framework, and
# re-execs elements-native-executor.py with them as explicit arguments.
#
# The harness's own two arguments arrive here as this launcher's arguments and
# are passed straight through, which is why the pass-through below is part of
# the contract rather than a convenience for running it by hand.
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
#   ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID     the development network identity
#                                           of the chain booted, as 64 hex
#                                           digits (required)
#   ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP   the integration tip the operator
#                                           intended to run (optional,
#                                           ADR-018)
#   ELEMENTS_NATIVE_EXECUTOR_UPSTREAM_BASE  the upstream base that tip
#                                           derives from (optional, ADR-018)
#   ELEMENTS_NATIVE_EXECUTOR_LOCAL_TOPICS   space-separated local topic
#                                           branches folded into that tip
#                                           (optional, ADR-018)
#   ELEMENTS_NATIVE_EXECUTOR_ENABLE_WALLET  set to any non-empty value to
#                                           boot the node with its wallet
#                                           enabled, which the Guide-12
#                                           section 16.2 operation lane
#                                           needs (optional). A wallet
#                                           NAME is not a credential and
#                                           none is named here; the
#                                           adapter creates a disposable
#                                           wallet on a disposable chain
#   ELEMENTS_NATIVE_EXECUTOR_PYTHON         python interpreter (default
#                                           python3)
#   ELEMENTS_NATIVE_EXECUTOR_LAUNCH_PREFIX  word-split command placed in
#                                           front of the interpreter, for
#                                           hosts where the node binaries
#                                           only run inside a container
#                                           (optional)
#
# Any further arguments given to this launcher are appended to the adapter's
# own. That is how the harness's --output and --elements-output reach the
# adapter, and it is also how an operator running this by hand supplies them;
# the adapter requires both and starts without neither.
#
# Stdout is protocol data. The adapter writes nothing at all on stderr, so
# this launcher's own refusal below is the only thing that ever appears
# there -- and it appears before the adapter has been executed.
set -eu

here="$(cd "$(dirname "$0")" && pwd)"
adapter="$here/elements-native-executor.py"

missing=""
for name in ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD \
            ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI \
            ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK \
            ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID; do
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
       --network-id "$ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID" \
       "$@"

# ADR-018 provenance. Each is an operator declaration of what was meant to
# run; none is derived from a working tree, and none stands in for the
# revision the node binary reports about itself.
if [ -n "${ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP:-}" ]; then
  set -- --intended-executed-tip "$ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP" "$@"
fi
if [ -n "${ELEMENTS_NATIVE_EXECUTOR_UPSTREAM_BASE:-}" ]; then
  set -- --upstream-base "$ELEMENTS_NATIVE_EXECUTOR_UPSTREAM_BASE" "$@"
fi
for topic in ${ELEMENTS_NATIVE_EXECUTOR_LOCAL_TOPICS:-}; do
  set -- --included-local-topic "$topic" "$@"
done

if [ -n "${ELEMENTS_NATIVE_EXECUTOR_ENABLE_WALLET:-}" ]; then
  set -- --enable-wallet "$@"
fi

python="${ELEMENTS_NATIVE_EXECUTOR_PYTHON:-python3}"

# Deliberately word-split: the prefix is a command, not one path.
# shellcheck disable=SC2086
exec ${ELEMENTS_NATIVE_EXECUTOR_LAUNCH_PREFIX:-} "$python" "$adapter" "$@"
