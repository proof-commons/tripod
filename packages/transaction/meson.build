# Rust-source census of the transaction crate (ADR-014): explicit and
# hand-managed — a new source file must be listed here, or the census
# audit fails the build. The parent meson.build accumulates this list
# into its role group.
transaction_source_files = files(
  'src/abi.rs',
  'src/bytes.rs',
  'src/construct.rs',
  'src/error.rs',
  'src/lib.rs',
  'src/live_abi.rs',
  'src/live_accepted.rs',
  'src/live_census.rs',
  'src/live_construct.rs',
  'src/live_finalize.rs',
  'src/live_handoff.rs',
  'src/live_materialize.rs',
  'src/live_message.rs',
  'src/live_private.rs',
  'src/live_request.rs',
  'src/live_signing.rs',
  'src/live_taproot.rs',
  'src/operator_membership.rs',
  'src/operator_right.rs',
  'src/operator_signing.rs',
  'src/request.rs',
  'src/script_path_signing.rs',
  'src/sponsor.rs',
  'src/state_abi.rs',
  'src/state_construct.rs',
  'src/state_finalize.rs',
  'src/state_request.rs',
  'src/state_view.rs',
  'src/synthetic.rs',
  'src/taproot.rs',
  'src/tests/abi_tests.rs',
  'src/tests/construction_tests.rs',
  'src/tests/ctf_materialize_tests.rs',
  'src/tests/ctf_preflight_tests.rs',
  'src/tests/ctf_serialization_tests.rs',
  'src/tests/encoding_tests.rs',
  'src/tests/guide13_preflight_tests.rs',
  'src/tests/live_abi_tests.rs',
  'src/tests/live_accepted_tests.rs',
  'src/tests/live_census_tests.rs',
  'src/tests/live_construction_tests.rs',
  'src/tests/live_handoff_tests.rs',
  'src/tests/live_request_tests.rs',
  'src/tests/live_signing_tests.rs',
  'src/tests/live_support.rs',
  'src/tests/mod.rs',
  'src/tests/operator_membership_tests.rs',
  'src/tests/operator_right_tests.rs',
  'src/tests/operator_signing_tests.rs',
  'src/tests/refusal_census_tests.rs',
  'src/tests/state_abi_tests.rs',
  'src/tests/state_construct_tests.rs',
  'src/tests/state_finalize_tests.rs',
  'src/tests/state_request_tests.rs',
  'src/tests/state_support.rs',
  'src/tests/state_view_tests.rs',
  'src/tests/taproot_tests.rs',
  'src/view.rs',
)

# Crate documentation (DOC owner).
transaction_doc_files = files('README.md')

# Same-typed files deliberately outside the crate-source census
# (ADR-014). The crate declares none yet; the empty list keeps the
# parent's role composition uniform across crates.
transaction_excluded_files = files()
