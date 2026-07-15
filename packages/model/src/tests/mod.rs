//! Executable verification suite — shared-kernel and branch-scenario
//! tests.
//!
//! Implements `´test:verification:fixtures´`,
//! `´test:verification:kernel´`,
//! `´test:verification:open-asset-immunity´`,
//! `´test:verification:canonical-attestation-serialization´`,
//! `´test:verification:exact-flow-destruction´`,
//! `´test:verification:open-flow-partition´`,
//! `´test:verification:scenario-fixtures´`,
//! `´test:verification:request-lifecycle´`,
//! `´test:verification:admission´`,
//! `´test:verification:cycle´`,
//! `´test:verification:distribution-fixtures´`,
//! `´test:verification:settlement´`,
//! `´test:verification:transfer´`,
//! `´test:verification:receipt-relabel´`,
//! `´test:verification:advanced-fixtures´`,
//! `´test:verification:redemption´`,
//! `´test:verification:burn-attestation´`,
//! `´test:verification:ash-clear´`,
//! `´test:verification:root-provenance´`,
//! `´test:verification:residue-noninterference´`,
//! `´test:verification:attestation-history-fixture´`,
//! `´test:verification:attestation-reorg-reprojection´`,
//! `´test:verification:canonical-serialization-rejection´`,
//! `´test:verification:corruption-fixtures´`,
//! `´test:verification:canonical-closure´`,
//! `´test:verification:distribution-bijection´`,
//! `´test:verification:receipt-class-accounting´`,
//! `´test:verification:sweepability´`,
//! `´test:verification:residue-noninterference-branches´`,
//! `´test:verification:native-fee-auction´`,
//! `´test:verification:no-accumulator´`,
//! `´test:verification:contended-state-fixture´`,
//! `´test:verification:native-fee-auction-two-valid´`,
//! `´test:verification:certificate-corruption-fixtures´`,
//! `´test:verification:root-certificate-faults´`,
//! `´test:verification:canonical-flow-faults´`,
//! `´test:verification:indexer-conformance´`,
//! `´test:verification:property-seed-strategy´`,
//! `´test:verification:property-traces´`,
//! `´test:verification:guard-listing-weld´`,
//! `´test:verification:model-label-register´`, and
//! `´test:verification:property-maintenance´`.
//!
//! Fixtures not yet used here are reserved for the remaining
//! branch-level installments.
//!
//! Evidence boundary: the executable oracle validates the abstract
//! authorization relation represented by `SignerSet`; exact Elements
//! signature bytes and sighash flags remain compiler/deployment
//! evidence. The indexer tests derive event provenance from transition
//! certificates; a separate differential suite must derive the same
//! events from emitted Elements transactions.
//!
//! # Test catalogue: categories and evidence classes
//!
//! Each test module belongs to one category, and each category makes
//! one evidence-class claim. A green test never upgrades its evidence
//! class; the generated tables
//! `[tbl:manifest:input-authorization-evidence]` and
//! `[tbl:manifest:operation-authorization-evidence]` bind the same
//! discipline to authorization claims.
//!
//! | Category | Modules | Evidence class | Claim |
//! |---|---|---|---|
//! | kernel-structural | `kernel_tests`, `exact_flow_tests`, `open_flow_tests`, `canonical_flow_fault_tests` | executable-test-backed (structural) | conservation, exact witness partition, shape — **not** authorization |
//! | operations | `request_lifecycle_tests`, `admission_tests`, `cycle_tests`, `settlement_tests`, `transfer_tests`, `redemption_tests`, `relabel_tests`, `burn_attestation_tests`, `ash_clear_tests`, `authorization_tests` | executable-test-backed (model authorization) | branch semantics plus signer-set authorization |
//! | invariant | `class_accounting_tests`, `canonical_closure_tests`, `clause_weld_tests`, `guard_listing_weld_tests`, `distribution_bijection_tests`, `root_provenance_tests`, `root_certificate_fault_tests`, `corruption_fixtures`, `bound_conformance_tests` | executable-test-backed | invariant and corruption rejection |
//! | assumption-backed | scarcity/no-forgery statements inside invariant tests | assumption-backed | canonical assets cannot be externally created (Elements conservation dependency) |
//! | deployment | (later) | deployment-tested | raw Elements transactions, sighash behavior |
#![allow(dead_code, clippy::wildcard_imports)]

pub mod advanced_fixtures;
pub mod attestation_history_fixture;
pub mod corruption_fixtures;
pub mod distribution_fixtures;
pub mod property_strategy;
pub mod scenario_fixtures;
pub mod test_fixtures;

mod admission_tests;
mod ash_clear_tests;
mod authorization_tests;
mod bound_conformance_tests;
mod burn_attestation_tests;
mod canonical_closure_tests;
mod class_accounting_tests;
mod clause_weld_tests;
mod cycle_tests;
mod distribution_bijection_tests;
mod exact_flow_tests;
mod guard_listing_weld_tests;
mod kernel_tests;
mod open_asset_immunity_tests;
mod open_flow_tests;
mod redemption_tests;
mod relabel_tests;
mod request_lifecycle_tests;
mod root_certificate_fault_tests;
mod root_provenance_tests;
mod settlement_tests;
mod stronger_fee_auction_tests;
mod transfer_tests;

mod active_backing_cap_tests;
mod canonical_flow_fault_tests;
mod fee_auction_tests;
