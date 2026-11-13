#!/bin/sh
# Test-only protocol adapter. Tests create aliases, never rewrite this inode.
# Its own alias names the behavior; the sibling alias names the compiled mock.
directory=${0%/*}
behavior=${0##*/}
behavior=${behavior#executor-}
behavior=${behavior%.sh}
if [ "$behavior" = raw-stderr ]; then
    printf '%s\n' 'MOCK_EXECUTOR_STDERR_THAT_MUST_NOT_BE_RELAYED' >&2
    behavior=answer-from-census
fi
exec "$directory/mock-native-executor" --behavior "$behavior" "$@"
