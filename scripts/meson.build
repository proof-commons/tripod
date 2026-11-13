# Script-tree census (ADR-014, ADR-023): the Python sources here are
# label subjects and must be listed, or the census audit fails the
# build. The shell sources beside them are not scanned and are removed
# by the categorical exclusion pattern in the parent meson.build.
script_source_files = files(
  'cargo-test-warmup.py',
  'keyed-cost-kranich.py',
  'check-vector-test-targets.py',
  'check-workspace-tracing.py',
  'ci-timing-report.py',
  'diagnose-taproot-issuance-rangeproof-digest.py',
  'diagnose-taproot-output-witness-digest.py',
  'elements-native-executor.py',
  'elements-observer-capture.py',
  'elements-observer-read.py',
  'executor_supervision.py',
  'run-conservation-matrix.py',
  'run-fresh-process-lifecycle.py',
  'run-normalization-matrix.py',
  'test-elements-observer-contract.py',
  'test-cargo-lanes.py',
  'test-executor-boundaries.py',
  'test-executor-classification.py',
  'test-executor-request-comparisons.py',
  'test-keyed-final-capture-wrappers.py',
)

# Nested shell helpers are explicit census exclusions, not label subjects.
script_excluded_files = files(
  'keyed-final-capture/capture-prepare.sh',
  'keyed-final-capture/capture-unit.sh',
  'keyed-final-capture/run-one.sh',
)

test(
  'keyed-final-capture-wrappers',
  find_program('python3'),
  args: [files('test-keyed-final-capture-wrappers.py')],
  suite: 'lint',
  timeout: 60,
)

test(
  'compact-native-capture-contract',
  find_program('compact-native-capture-test.sh'),
  args: [meson.project_source_root()],
  suite: 'lint',
  timeout: 120,
)

test(
  'compact-native-funding-contract',
  find_program('sh'),
  args: [files('compact-native-funding-test.sh'), meson.project_source_root()],
  suite: 'lint',
  timeout: 120,
)
