# Rust-source census of the vectors crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build. The parent meson.build accumulates this list
# into its role group.
vectors_source_files = files(
  'src/abi_validation.rs',
  'src/bundle.rs',
  'src/comparison.rs',
  'src/divergence.rs',
  'src/error.rs',
  'src/first_party.rs',
  'src/fixture.rs',
  'src/confidential_materializer.rs',
  'src/confidential_predecessor.rs',
  'src/lib.rs',
  'src/live_capability.rs',
  'src/live_closeout.rs',
  'src/live_comparison.rs',
  'src/live_conservation_negatives.rs',
  'src/live_disclosure.rs',
  'src/live_evidence.rs',
  'src/live_fault_discharge.rs',
  'src/live_first_party.rs',
  'src/live_keypath_probe.rs',
  'src/live_measurements.rs',
  'src/live_minimality_report.rs',
  'src/live_multi_shapes.rs',
  'src/live_native.rs',
  'src/live_owner_observation.rs',
  'src/live_pairs.rs',
  'src/live_plan.rs',
  'src/live_private_restart.rs',
  'src/live_proof_bearing_observation.rs',
  'src/live_report.rs',
  'src/live_resource_report.rs',
  'src/live_restart.rs',
  'src/live_resources.rs',
  'src/live_roles.rs',
  'src/live_safety.rs',
  'src/materialize.rs',
  'src/matrix.rs',
  'src/mutation.rs',
  'src/operation.rs',
  'src/plan.rs',
  'src/projection.rs',
  'src/render.rs',
  'src/report.rs',
  'src/resource_study.rs',
  'src/subject.rs',
  'src/violation.rs',
)

# Crate documentation (DOC owner).
vectors_doc_files = files('README.md')

# Same-typed files deliberately outside the crate-source census
# (ADR-014): integration-test sources are cargo test inputs, not label
# subjects. The census audit accepts the exclusion only because it is
# declared here.
vectors_excluded_files = files(
  'tests/compact_ash_operation.rs',
  'tests/ctf_materializer.rs',
  'tests/guide13_live_native.rs',
  'tests/guide13_live_resources.rs',
  'tests/guide13_live_transfer.rs',
  'tests/guide13_preflight.rs',
  'tests/guide13_sponsor_signing.rs',
  'tests/sighash_verdict.rs',
)
