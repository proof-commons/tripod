//! The region classifier, the agreement census, and the origin-typed
//! commitment comparison.
//!
//! # Convention
//!
//! Running property checks over first-party values. Nothing here reaches
//! a target: the decoded transactions are stated by hand, and what is
//! under test is what the boundary does with them.

use crate::confidential_funding::{
    DecodedAssetField, DecodedFundingOutput, DecodedFundingTransaction, DecodedValueField,
    ReadbackDerivedOutput,
};
use crate::confidential_record::{
    AgreementOrigin, AgreementRefusal, CandidateFundingNonClaim, CanonicalFundingExclusion,
    FundingAgreementField, FundingOutputForm, FundingRegion, NonProtocolFundingRegion,
    OutputAgreementCensus, ReadBackCommitment, RecomputedCommitment, RegionClassificationRefusal,
    classify_funding_members, compare_commitments, parities_admitted, prefix_admitted_at,
};
use crate::protocol::WireOutpoint;

/// The protocol asset, in the target's own printed spelling.
fn protocol() -> String {
    "aa".repeat(32)
}

/// A published disposable asset identity, in commitment order.
const ASSET: [u8; 32] = [0x3b; 32];

/// One committed value field of the right width.
fn commitment(prefix: u8) -> Vec<u8> {
    let mut bytes = vec![0x11_u8; 33];
    bytes[0] = prefix;
    bytes
}

/// One decoded protocol output.
fn protocol_output(asset: &str, program: &[u8]) -> DecodedFundingOutput {
    DecodedFundingOutput {
        asset: DecodedAssetField::Explicit(asset.to_owned()),
        value: DecodedValueField::Commitment(commitment(8)),
        nonce: vec![0x02; 33],
        program: program.to_vec(),
        surjection_proof: Vec::new(),
        rangeproof: vec![0x7a; 64],
    }
}

/// One decoded member outside the protocol region.
fn policy_member(asset: &str, program: &[u8]) -> DecodedFundingOutput {
    DecodedFundingOutput {
        asset: DecodedAssetField::Explicit(asset.to_owned()),
        value: DecodedValueField::Explicit(1_000),
        nonce: Vec::new(),
        program: program.to_vec(),
        surjection_proof: Vec::new(),
        rangeproof: Vec::new(),
    }
}

/// One transaction with two protocol members, one change, and one fee.
fn transaction(policy: &str) -> DecodedFundingTransaction {
    DecodedFundingTransaction {
        transaction_id: "11".repeat(32),
        witness_transaction_id: "22".repeat(32),
        outputs: vec![
            protocol_output(protocol().as_str(), &[0x51]),
            protocol_output(protocol().as_str(), &[0x52]),
            policy_member(policy, &[0x51]),
            policy_member(policy, &[]),
        ],
    }
}

#[test]
fn every_member_is_placed_in_a_region_and_the_fee_is_named() {
    let policy = "bb".repeat(32);
    let regions = classify_funding_members(&transaction(&policy), protocol().as_str(), 2)
        .expect("every member places");
    // Exhaustive by construction: one entry per member, in the
    // transaction's own order.
    assert_eq!(regions.len(), 4);
    assert_eq!(regions[0], FundingRegion::Protocol { position: 0 });
    assert_eq!(regions[1], FundingRegion::Protocol { position: 1 });
    assert_eq!(
        regions[2],
        FundingRegion::NonProtocol(NonProtocolFundingRegion::PolicyChange)
    );
    assert_eq!(
        regions[3],
        FundingRegion::NonProtocol(NonProtocolFundingRegion::PolicyFee)
    );
}

#[test]
fn a_member_outside_the_protocol_region_may_not_carry_the_protocol_asset() {
    // The whole reason the region exists: a fee paid in the protocol
    // asset would make the two-output balance an argument about which
    // region a member was put in.
    let mut subject = transaction(protocol().as_str());
    subject.outputs[2] = policy_member(protocol().as_str(), &[0x51]);
    assert_eq!(
        classify_funding_members(&subject, protocol().as_str(), 2).expect_err("it refuses"),
        RegionClassificationRefusal::ProtocolAssetOutsideRegion { index: 2 }
    );
}

#[test]
fn a_non_protocol_member_may_carry_neither_a_commitment_nor_a_proof() {
    let policy = "bb".repeat(32);
    let mut committed = transaction(&policy);
    committed.outputs[2].value = DecodedValueField::Commitment(commitment(8));
    assert_eq!(
        classify_funding_members(&committed, protocol().as_str(), 2).expect_err("it refuses"),
        RegionClassificationRefusal::NonProtocolMemberNotExplicit { index: 2 }
    );
    let mut proved = transaction(&policy);
    proved.outputs[3].rangeproof = vec![0x01];
    assert_eq!(
        classify_funding_members(&proved, protocol().as_str(), 2).expect_err("it refuses"),
        RegionClassificationRefusal::NonProtocolMemberCarriesProof { index: 3 }
    );
}

#[test]
fn a_blinded_sponsor_coin_trips_the_explicitness_clause_before_the_proof_clause() {
    // A BLINDED SPONSOR COIN is the shape a confidential sponsor value
    // needs its funding transaction to create: the reserve asset
    // explicit, because the isolation fragment introspects it, and the
    // value committed with the range proof that a committed value
    // requires. It is non-protocol by asset, so it lands outside the
    // protocol region and meets the two clauses that guard it.
    //
    // It trips BOTH, and which one fires first is the finding. The
    // explicitness clause is checked before the proof clause, so a
    // reader who repaired only the refusal they saw would fix the
    // commitment and be met immediately by the proof — the second
    // refusal being MASKED by the first rather than absent.
    //
    // Recorded as a run rather than as a reading of the branch order,
    // because the order is what a repair has to plan around and a
    // reordering of these two clauses would otherwise change the
    // finding silently.
    let policy = "bb".repeat(32);
    let mut blinded = transaction(&policy);
    blinded.outputs[2] = DecodedFundingOutput {
        asset: DecodedAssetField::Explicit(policy.clone()),
        value: DecodedValueField::Commitment(commitment(8)),
        nonce: vec![0x02; 33],
        program: vec![0x51],
        // Empty, as it must be: a surjection proof is required exactly
        // when the ASSET is committed, and this one is not.
        surjection_proof: Vec::new(),
        rangeproof: vec![0x7a; 64],
    };
    assert_eq!(
        classify_funding_members(&blinded, protocol().as_str(), 2).expect_err("it refuses"),
        RegionClassificationRefusal::NonProtocolMemberNotExplicit { index: 2 },
        "the explicitness clause is no longer the first one a blinded sponsor coin meets",
    );

    // The masked one, shown by removing only what the first clause
    // objects to. Nothing else about the member moves, so the second
    // refusal is attributable to the proof and to nothing else.
    let mut without_commitment = blinded;
    without_commitment.outputs[2].value = DecodedValueField::Explicit(1_250);
    assert_eq!(
        classify_funding_members(&without_commitment, protocol().as_str(), 2)
            .expect_err("it refuses"),
        RegionClassificationRefusal::NonProtocolMemberCarriesProof { index: 2 },
        "the proof clause was not the second obstacle after all",
    );

    // And the region vocabulary has no member for it either way: the
    // classifier decides a non-protocol member's region by whether its
    // program is empty, so a sponsor coin can only ever be read as
    // policy change. Admitting a blinded sponsor coin therefore needs a
    // region member as well as the two clauses, which is why this is a
    // vocabulary change rather than a relaxation.
    assert_eq!(
        NonProtocolFundingRegion::PolicyChange.to_string(),
        "policy-asset change",
    );
}

#[test]
fn a_protocol_position_must_carry_the_asset_and_a_committed_value() {
    let policy = "bb".repeat(32);
    let mut wrong_asset = transaction(&policy);
    wrong_asset.outputs[1] = protocol_output(&policy, &[0x52]);
    assert_eq!(
        classify_funding_members(&wrong_asset, protocol().as_str(), 2).expect_err("it refuses"),
        RegionClassificationRefusal::ProtocolAssetAbsent { position: 1 }
    );
    let mut explicit = transaction(&policy);
    explicit.outputs[0].value = DecodedValueField::Explicit(7);
    assert_eq!(
        classify_funding_members(&explicit, protocol().as_str(), 2).expect_err("it refuses"),
        RegionClassificationRefusal::ProtocolValueNotCommitted { position: 0 }
    );
    let mut short = transaction(&policy);
    short.outputs.truncate(1);
    assert_eq!(
        classify_funding_members(&short, protocol().as_str(), 2).expect_err("it refuses"),
        RegionClassificationRefusal::ProtocolCountShort {
            required: 2,
            observed: 1
        }
    );
}

#[test]
fn an_independent_origin_cannot_self_attest() {
    // A comparison of a value with itself is not evidence, and it is
    // refused where the census is assembled rather than reported as
    // agreement.
    let fields = crate::confidential_record::stated_agreements(&[(
        FundingAgreementField::Asset,
        AgreementOrigin::ReadBack,
        AgreementOrigin::ReadBack,
        true,
    )]);
    assert_eq!(
        OutputAgreementCensus::assemble(0, fields).expect_err("it refuses"),
        AgreementRefusal::SelfAttestation {
            field: FundingAgreementField::Asset
        }
    );
}

#[test]
fn the_census_must_carry_every_member() {
    let fields = crate::confidential_record::stated_agreements(&[(
        FundingAgreementField::Asset,
        AgreementOrigin::Recomputed,
        AgreementOrigin::ReadBack,
        true,
    )]);
    assert_eq!(
        OutputAgreementCensus::assemble(3, fields).expect_err("it refuses"),
        AgreementRefusal::CensusIncomplete { output: 3 }
    );
}

#[test]
fn a_disagreeing_field_names_itself() {
    let mut stated: Vec<_> = FundingAgreementField::ALL
        .into_iter()
        .map(|field| {
            (
                field,
                AgreementOrigin::Recomputed,
                AgreementOrigin::ReadBack,
                true,
            )
        })
        .collect();
    stated[2].3 = false;
    let census =
        OutputAgreementCensus::assemble(1, crate::confidential_record::stated_agreements(&stated))
            .expect("it assembles");
    assert_eq!(
        census.require_agreement().expect_err("it refuses"),
        AgreementRefusal::FieldDisagrees {
            output: 1,
            field: FundingAgreementField::Parity
        }
    );
}

#[test]
fn a_recomputation_is_compared_only_against_a_readback() {
    let recomputed =
        RecomputedCommitment::recompute(&ASSET, 700_000_000, &[0x0c; 32]).expect("it recomputes");
    let observed = ReadBackCommitment::from_readback(&ReadbackDerivedOutput {
        outpoint: WireOutpoint {
            txid: "11".repeat(32),
            vout: 0,
        },
        explicit_asset: protocol().as_str().to_owned(),
        value_commitment: recomputed.bytes().to_vec(),
        nonce: vec![0x02; 33],
        output_program: vec![0x51],
        rangeproof: vec![0x7a; 64],
    })
    .expect("the width is the committed one");
    let check = compare_commitments(recomputed, observed);
    assert!(check.agrees());
    // The signature is the guarantee: two different types, so there is
    // no way to spell a comparison of a readback with a readback.
    assert_eq!(check.recomputed().bytes(), check.observed().bytes());
}

#[test]
fn the_form_census_and_the_two_exhaustive_vocabularies_are_complete() {
    assert_eq!(FundingOutputForm::complete().len(), 4);
    assert_eq!(FundingAgreementField::ALL.len(), 8);
    assert_eq!(CanonicalFundingExclusion::ALL.len(), 9);
    assert_eq!(CandidateFundingNonClaim::ALL.len(), 10);
}

/// The reviewed value field, whose admitted committed prefixes the arity
/// rule reads from rather than restating.
fn value_encoding() -> target_elements::ConfidentialFieldEncoding {
    target_elements::reviewed_confidential_review_facts().value()
}

#[test]
fn a_two_output_record_still_demands_the_admitted_pair_in_fixed_order() {
    let encoding = value_encoding();
    let (first, second) = encoding.committed_prefixes();

    // The dual-parity rule, unchanged: one of each, in the encoder's own
    // order. This is the case the parity search was built to discriminate,
    // and the arity generalization must not have weakened it.
    assert!(parities_admitted(&encoding, &[first, second]));
    assert!(!parities_admitted(&encoding, &[second, first]));
    assert!(!parities_admitted(&encoding, &[first, first]));
    assert!(!parities_admitted(&encoding, &[second, second]));
}

#[test]
fn a_record_wider_than_two_outputs_is_read_and_not_panicked_on() {
    let encoding = value_encoding();
    let (first, second) = encoding.committed_prefixes();

    // The regression. The rule used to be a two-element array indexed by
    // output index, so reaching a third output was an out-of-bounds panic
    // rather than a verdict — and a panic is not a refusal any caller can
    // catch, report, or reason about. Every one of these calls indexes
    // past the pair.
    assert!(parities_admitted(&encoding, &[first, second, first]));
    assert!(parities_admitted(&encoding, &[second, second, second]));
    assert!(parities_admitted(
        &encoding,
        &[first, first, second, second, first]
    ));

    // Membership is still a rule and not an absence of one: a prefix
    // outside the admitted pair is refused at any width.
    let outside = first ^ 0x40;
    assert!(outside != first && outside != second);
    assert!(!parities_admitted(&encoding, &[first, second, outside]));

    // And the per-output reading agrees with the sequence reading at the
    // index that used to panic, which is what makes the summary check and
    // the per-output check one rule rather than two.
    assert!(prefix_admitted_at(&encoding, first, 2, 3));
    assert!(prefix_admitted_at(&encoding, second, 2, 3));
    assert!(!prefix_admitted_at(&encoding, outside, 2, 3));
}

#[test]
fn a_single_output_record_admits_either_parity_because_its_blinder_is_forced() {
    let encoding = value_encoding();
    let (first, second) = encoding.committed_prefixes();

    // A lone output's blinder is not chosen — it is forced to the input
    // blinder sum — so the prefix it carries is whichever that forced
    // blinder yields. Demanding the FIRST prefix, which is what indexing a
    // fixed pair silently did, was a rule nobody had stated.
    assert!(parities_admitted(&encoding, &[first]));
    assert!(parities_admitted(&encoding, &[second]));

    let outside = second ^ 0x40;
    assert!(outside != first && outside != second);
    assert!(!parities_admitted(&encoding, &[outside]));
}

#[test]
fn the_records_arity_rule_agrees_with_the_registrys_parity_search() {
    let encoding = value_encoding();
    let (first, second) = encoding.committed_prefixes();

    // The two halves of one rule live in two files — the registry decides
    // which openings to derive, the record decides whether a mined
    // readback satisfies the same contract — so the agreement is checked
    // rather than assumed. Every width but two is membership on both
    // sides; two is fixed order on both sides.
    for width in [1_usize, 2, 3, 4] {
        let fixed_order = width == 2;
        assert_eq!(
            fixed_order,
            !prefix_admitted_at(&encoding, second, 0, width),
            "width {width} disagrees about the first position"
        );
        assert!(
            prefix_admitted_at(&encoding, first, 0, width),
            "width {width} refuses the first prefix at the first position"
        );
    }
}
