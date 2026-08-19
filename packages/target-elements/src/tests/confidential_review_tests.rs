//! Tests over the reviewed confidential-value facts.
//!
//! These pin the facts the Guide-11 target review established from
//! source. A test here failing means either the transcription drifted
//! or the reviewed target changed; neither is a test to relax.

use crate::confidential::{
    CommitmentTermRoles, ConservationForm, OpeningBlocker, PointParityConvention, ScalarByteOrder,
    reviewed_confidential_review_facts, reviewed_confidential_values,
};
use crate::encoding::EncodingClass;
use crate::opcode::OpcodeId;

#[test]
fn explicit_value_is_a_prefix_and_eight_bytes() {
    let value = reviewed_confidential_review_facts().value();
    assert_eq!(value.explicit_width(), 9);
    assert_eq!(value.explicit_prefix(), 1);
}

#[test]
fn every_confidential_field_commits_in_thirty_three_bytes() {
    let facts = reviewed_confidential_review_facts();
    for field in [facts.value(), facts.asset(), facts.nonce()] {
        assert_eq!(field.committed_width(), 33);
    }
}

#[test]
fn explicit_asset_is_as_wide_as_its_commitment() {
    // The asset field is the one whose two forms coincide in width,
    // which is why a width test alone cannot tell them apart and the
    // prefix is load-bearing.
    let asset = reviewed_confidential_review_facts().asset();
    assert_eq!(asset.explicit_width(), asset.committed_width());
    assert_ne!(asset.explicit_prefix(), asset.committed_prefixes().0);
}

#[test]
fn committed_prefixes_are_distinct_across_the_fields() {
    let facts = reviewed_confidential_review_facts();
    assert_eq!(facts.value().committed_prefixes(), (8, 9));
    assert_eq!(facts.asset().committed_prefixes(), (10, 11));
    assert_eq!(facts.nonce().committed_prefixes(), (2, 3));
}

#[test]
fn committed_prefix_pairs_differ_in_the_low_bit_only() {
    // The encoder writes a per-field constant exclusive-or one bit
    // about the y coordinate, so the pair is always adjacent. Which
    // bit it is differs by field, which is why the pair is tested here
    // and the convention separately below.
    let facts = reviewed_confidential_review_facts();
    for field in [facts.value(), facts.asset(), facts.nonce()] {
        let (square, non_square) = field.committed_prefixes();
        assert_eq!(square ^ non_square, 1);
        assert_eq!(square & 1, 0);
    }
}

#[test]
fn confidential_encodings_record_squareness_not_oddness() {
    // The fact that keeps a taproot-shaped pattern from being carried
    // over by analogy.
    let facts = reviewed_confidential_review_facts();
    for field in [facts.value(), facts.asset()] {
        assert_eq!(field.parity(), PointParityConvention::QuadraticResidue);
        assert_ne!(field.parity(), PointParityConvention::CompressedOddness);
    }
    // The nonce is the table's own instance of the other convention:
    // the target transports the point rather than committing to it, so
    // it is written with the compressed pair the curve primitives
    // accept. A table that recorded squareness here would erase the
    // disagreement the opening blockers rest on.
    assert_eq!(
        facts.nonce().parity(),
        PointParityConvention::CompressedOddness
    );
}

#[test]
fn a_field_admits_only_its_own_prefixes() {
    let value = reviewed_confidential_review_facts().value();
    assert!(value.admits_prefix(0));
    assert!(value.admits_prefix(1));
    assert!(value.admits_prefix(8));
    assert!(value.admits_prefix(9));
    // The asset prefixes are not value prefixes.
    assert!(!value.admits_prefix(10));
    assert!(!value.admits_prefix(11));
    // Nor is a compressed public-key prefix.
    assert!(!value.admits_prefix(2));
}

#[test]
fn the_generator_recipe_needs_two_curve_maps_and_an_addition() {
    let generator = reviewed_confidential_review_facts().generator();
    assert_eq!(generator.identifier_width(), 32);
    assert_eq!(generator.hash_prefix_width(), 16);
    assert_eq!(generator.curve_map_evaluations(), 2);
    assert_eq!(generator.point_additions(), 1);
    assert!(generator.blinded_form_adds_base_multiple());
}

#[test]
fn the_relation_puts_the_blinder_on_the_base_point() {
    let relation = reviewed_confidential_review_facts().relation();
    assert_eq!(
        relation.roles(),
        CommitmentTermRoles::BlindOnBaseAmountOnAssetGenerator
    );
}

#[test]
fn the_opening_scalar_is_thirty_two_bytes_big_endian() {
    let relation = reviewed_confidential_review_facts().relation();
    assert_eq!(relation.scalar_width(), 32);
    assert_eq!(relation.scalar_order(), ScalarByteOrder::BigEndian);
}

#[test]
fn a_zero_scalar_is_admitted_and_an_overflowing_one_is_not() {
    let relation = reviewed_confidential_review_facts().relation();
    // Consensus commits every explicit amount under a zero blinder, so
    // refusing the zero scalar would contradict the reviewed source.
    assert!(relation.admits_zero_scalar());
    assert!(relation.rejects_scalar_at_or_above_group_order());
    assert!(relation.rejects_identity_result());
}

#[test]
fn conservation_is_an_exact_tally() {
    let facts = reviewed_confidential_review_facts();
    let conservation = facts.conservation();
    assert_eq!(conservation.form(), ConservationForm::ExactTallyToIdentity);
    assert_ne!(conservation.form(), ConservationForm::ExcessCarried);
    assert!(conservation.explicit_values_join_as_zero_blinded());
    assert!(conservation.issuance_contributes_to_input_side());
    assert!(conservation.unspendable_zero_output_leaves_tally());
}

#[test]
fn proofs_are_required_exactly_for_the_blinded_encodings() {
    let facts = reviewed_confidential_review_facts();
    let conservation = facts.conservation();

    let range = conservation.proofs().range_proof_required_for();
    assert!(range.contains(&EncodingClass::ConfidentialValue));
    assert!(!range.contains(&EncodingClass::ExplicitValue));

    let surjection = conservation.proofs().surjection_proof_required_for();
    assert!(surjection.contains(&EncodingClass::ConfidentialAsset));
    assert!(!surjection.contains(&EncodingClass::ExplicitAsset));
}

#[test]
fn the_range_proof_binds_its_output_and_excludes_zero() {
    let facts = reviewed_confidential_review_facts();
    let proofs = facts.conservation().proofs();
    assert!(proofs.range_proof_binds_output_script());
    assert!(proofs.range_proof_excludes_zero_when_spendable());
}

#[test]
fn an_on_script_opening_is_not_reachable() {
    let facts = reviewed_confidential_review_facts();
    assert!(!facts.opening().reachable());
}

#[test]
fn every_reviewed_blocker_is_named() {
    let facts = reviewed_confidential_review_facts();
    let blockers = facts.opening().blockers();
    assert_eq!(blockers.len(), OpeningBlocker::ALL.len());
    for blocker in OpeningBlocker::ALL {
        assert!(blockers.contains(blocker));
    }
}

#[test]
fn the_candidate_primitives_are_listed_without_implying_a_pattern() {
    let facts = reviewed_confidential_review_facts();
    let candidates = facts.opening().candidate_primitives();
    // The curve primitives a candidate would build on exist.
    assert!(candidates.contains(&OpcodeId::EcMulScalarVerify));
    assert!(candidates.contains(&OpcodeId::TweakVerify));
    assert!(candidates.contains(&OpcodeId::Concatenate));
    // Their presence is nonetheless not reachability.
    assert!(!facts.opening().reachable());
    assert!(!facts.opening().blockers().is_empty());
}

#[test]
fn the_review_facts_do_not_contradict_the_capability_contract() {
    // The capability contract calls an authenticated opening
    // unsupported. The feasibility result must agree with it, and the
    // blockers are the reason rather than a separate opinion.
    let contract = reviewed_confidential_values();
    let facts = reviewed_confidential_review_facts();
    let unsupported = matches!(
        contract
            .states()
            .get(&crate::confidential::ConfidentialValueCapability::AuthenticatedOpening),
        Some(crate::confidential::ConfidentialCapabilityState::Unsupported)
    );
    assert!(unsupported);
    assert_eq!(unsupported, !facts.opening().reachable());
}

#[test]
fn the_facts_are_a_stable_value() {
    assert_eq!(
        reviewed_confidential_review_facts(),
        reviewed_confidential_review_facts()
    );
}
