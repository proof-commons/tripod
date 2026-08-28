//! Immutable values recorded by the historical fixture-digest-v1 campaign.
//!
//! This namespace changes ownership, not bytes. Each domain below is an
//! unchanged re-export of the declaration that originally recorded the value.
//! Current claims must use the validated native-v2/revision-7 corpus; callers
//! enter this namespace only when reconstructing or checking historical-v1
//! evidence.

/// Historical private-restart values and the complete typed V1 record.
pub mod private_restart {
    pub use crate::live_private_restart::run_of_record::{
        ACCEPTED_TXID, CONSUMED_COMMITMENT_PREFIX, HistoricalPrivateRestartAcceptedMember,
        HistoricalPrivateRestartRun, HistoricalPrivateRestartTwoAcceptanceLink,
        HistoricalPrivateRestartV1, ISSUED_ASSET, OUTPUT_WITNESS_PROOF_BYTES, PARITY_ACCEPTED_TXID,
        PARITY_CONSUMED_COMMITMENT_PREFIX, PARITY_SUCCESSOR_DIGEST, PREDECESSOR_DIGEST,
        RECEIPT_LEAVES, SUBMITTED_BYTES, SUCCESSOR_DIGEST, WALL_SECONDS, accepted,
        historical_private_restart_run, parity_accepted,
    };
}

/// Historical private multi-shape observations.
pub mod multi_shapes {
    pub use crate::live_multi_shapes::run_of_record::*;
}

/// Historical conservation control and refusal observations.
pub mod conservation_negatives {
    pub use crate::live_conservation_negatives::run_of_record::*;
}

/// Historical explicit-shape acceptance observations.
pub mod explicit_shapes {
    pub use crate::live_explicit_shapes::run_of_record::*;
}

/// Historical explicit witness-negative observations.
pub mod explicit_witness_negatives {
    pub use crate::live_explicit_shapes::witness_negatives_run_of_record::*;
}

/// Historical sponsored-shape observations.
pub mod sponsor_shapes {
    pub use crate::live_sponsor_shapes::sponsored_run_of_record::*;
}

/// Historical owner-signing refusal observations.
pub mod owner_signing_negatives {
    pub use crate::live_owner_signing_negatives::run_of_record::*;
}

/// Historical phase-A key-path probe observations.
pub mod keypath_probe {
    pub use crate::live_keypath_probe::run_of_record::*;
}

/// Historical phase-B key-path probe observations.
pub mod keypath_probe_phase_b {
    pub use crate::live_keypath_probe::run_of_record_phase_b::*;
}

/// Historical paired explicit/private observation.
pub mod pair_arc {
    pub use crate::live_pair_arc::run_of_record::*;
}

/// Historical selected-owner observation and its seven expected outcomes.
pub mod owner_observation {
    pub use crate::live_owner_observation::run_of_record::*;
}

/// Historical schema-1 proof-bearing records and observation identity.
pub mod proof_bearing {
    pub use crate::live_proof_bearing_observation::{
        PROOF_BEARING_OBSERVATION, PROOF_BEARING_RUN_OF_RECORD_SCHEMA_VERSION,
        ProofBearingRunOfRecord, ProofBearingRunOfRecordV2, T5_031_CONSTRUCTION_REFUSALS,
        construction_run_of_record_v2,
    };
}

/// Historical native transcript and resource observations.
pub mod native {
    pub use crate::live_native::historical_v1_transcript as transcript;
}

#[cfg(test)]
mod tests {
    const ACTIVE_CONSUMERS: &[(&str, &str)] = &[
        ("live_closeout.rs", include_str!("live_closeout.rs")),
        ("live_comparison.rs", include_str!("live_comparison.rs")),
        (
            "live_resource_report.rs",
            include_str!("live_resource_report.rs"),
        ),
        ("live_roles.rs", include_str!("live_roles.rs")),
        (
            "recorded_acceptance.rs",
            include_str!("recorded_acceptance.rs"),
        ),
        ("shape_census.rs", include_str!("shape_census.rs")),
        ("live_pairs.rs", include_str!("live_pairs.rs")),
        (
            "live_negative_half.rs",
            include_str!("live_negative_half.rs"),
        ),
        (
            "guide13_live_resources.rs",
            include_str!("../tests/guide13_live_resources.rs"),
        ),
        (
            "sighash_verdict.rs",
            include_str!("../tests/sighash_verdict.rs"),
        ),
    ];

    const RETIRED_ACTIVE_PATHS: &[&str] = &[
        "live_private_restart::run_of_record",
        "live_multi_shapes::run_of_record",
        "live_conservation_negatives::run_of_record",
        "live_explicit_shapes::run_of_record",
        "live_explicit_shapes::witness_negatives_run_of_record",
        "live_sponsor_shapes::sponsored_run_of_record",
        "live_owner_signing_negatives::run_of_record",
        "live_keypath_probe::run_of_record",
        "live_pair_arc::run_of_record",
        "live_owner_observation::run_of_record",
        "observed_run_of_record",
    ];

    fn recorded_digest(text: &str) -> [u8; 32] {
        let mut bytes = [0_u8; 32];
        let (pairs, remainder) = text.as_bytes().as_chunks::<2>();
        assert!(remainder.is_empty(), "a historical digest has odd width");
        for (slot, pair) in bytes.iter_mut().zip(pairs) {
            let digits = std::str::from_utf8(pair).expect("a historical digest is ASCII hex");
            *slot = u8::from_str_radix(digits, 16).expect("a historical digest is hexadecimal");
        }
        bytes
    }

    #[test]
    fn active_consumers_do_not_read_v1_origin_paths() {
        let stale = ACTIVE_CONSUMERS
            .iter()
            .flat_map(|(name, source)| {
                RETIRED_ACTIVE_PATHS
                    .iter()
                    .filter(|path| source.contains(**path))
                    .map(move |path| format!("{name}: {path}"))
            })
            .collect::<Vec<_>>();
        assert!(stale.is_empty(), "stale active v1 paths: {stale:#?}");
    }

    #[test]
    fn archival_reexports_are_the_original_values() {
        assert_eq!(
            super::private_restart::ACCEPTED_TXID,
            crate::live_private_restart::run_of_record::ACCEPTED_TXID,
        );
        assert_eq!(
            super::multi_shapes::OUTPUT_COUNTS,
            crate::live_multi_shapes::run_of_record::OUTPUT_COUNTS,
        );
        assert_eq!(
            super::conservation_negatives::WRONG_BLINDER_FIELD_RANGE,
            crate::live_conservation_negatives::run_of_record::WRONG_BLINDER_FIELD_RANGE,
        );
        assert_eq!(
            super::explicit_shapes::MAXIMUM_INPUTS_ACCEPTED_TXID,
            crate::live_explicit_shapes::run_of_record::MAXIMUM_INPUTS_ACCEPTED_TXID,
        );
        assert_eq!(
            super::sponsor_shapes::SPONSORED_ACCEPTED_TXID,
            crate::live_sponsor_shapes::sponsored_run_of_record::SPONSORED_ACCEPTED_TXID,
        );
        assert_eq!(
            super::owner_signing_negatives::CONTROL_ARRANGEMENT,
            crate::live_owner_signing_negatives::run_of_record::CONTROL_ARRANGEMENT,
        );
        assert_eq!(
            super::keypath_probe_phase_b::REFUSAL_DETAIL,
            crate::live_keypath_probe::run_of_record_phase_b::REFUSAL_DETAIL,
        );
        assert_eq!(
            super::pair_arc::TERMS_WITHHELD_BY_THE_PRIVATE_MEMBER,
            crate::live_pair_arc::run_of_record::TERMS_WITHHELD_BY_THE_PRIVATE_MEMBER,
        );
        assert_eq!(
            super::owner_observation::EXPECTED_CASE_OUTCOMES,
            crate::live_owner_observation::run_of_record::EXPECTED_CASE_OUTCOMES,
        );
    }

    #[test]
    fn conservation_q19_divergence_is_archival_only() {
        let corpus = crate::live_corpus_native_v2_r7::run_of_record()
            .expect("the reviewed corpus validates");
        let current = corpus
            .mint_ceremony("conservation-negatives")
            .expect("the conservation ceremony is present");
        assert_ne!(
            recorded_digest(super::conservation_negatives::PREDECESSOR_DIGEST),
            *current
                .fixture_digest("predecessor")
                .expect("the current predecessor digest is present"),
        );
        assert_ne!(
            recorded_digest(super::conservation_negatives::SUCCESSOR_DIGEST),
            *current
                .fixture_digest("successor")
                .expect("the current successor digest is present"),
        );
    }
}
