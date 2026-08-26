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
    let regions = classify_funding_members(&transaction(&policy), protocol().as_str(), 2, None)
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
        classify_funding_members(&subject, protocol().as_str(), 2, None).expect_err("it refuses"),
        RegionClassificationRefusal::ProtocolAssetOutsideRegion { index: 2 }
    );
}

#[test]
fn a_non_protocol_member_may_carry_neither_a_commitment_nor_a_proof() {
    let policy = "bb".repeat(32);
    let mut committed = transaction(&policy);
    committed.outputs[2].value = DecodedValueField::Commitment(commitment(8));
    assert_eq!(
        classify_funding_members(&committed, protocol().as_str(), 2, None).expect_err("it refuses"),
        RegionClassificationRefusal::NonProtocolMemberNotExplicit { index: 2 }
    );
    let mut proved = transaction(&policy);
    proved.outputs[3].rangeproof = vec![0x01];
    assert_eq!(
        classify_funding_members(&proved, protocol().as_str(), 2, None).expect_err("it refuses"),
        RegionClassificationRefusal::NonProtocolMemberCarriesProof { index: 3 }
    );
}

/// One blinded sponsor coin, as its funding transaction creates it.
///
/// The reserve asset EXPLICIT, because §10.7's isolation fragment
/// introspects it, and the value COMMITTED with the range proof a
/// committed value requires. The surjection proof is empty, as it must
/// be: the target requires one exactly when the ASSET is committed.
fn sponsor_coin(asset: &str, program: &[u8]) -> DecodedFundingOutput {
    DecodedFundingOutput {
        asset: DecodedAssetField::Explicit(asset.to_owned()),
        value: DecodedValueField::Commitment(commitment(8)),
        nonce: vec![0x02; 33],
        program: program.to_vec(),
        surjection_proof: Vec::new(),
        rangeproof: vec![0x7a; 64],
    }
}

#[test]
fn a_blinded_sponsor_coin_is_placed_in_its_own_region_once_its_program_is_declared() {
    // The repair of an observed defect, held here so that a reordering
    // of the classifier would fail rather than change the finding
    // silently.
    //
    // Before it, a blinded sponsor coin met one explicit-only rule that
    // every non-protocol member met, tripped it, and tripped it FIRST —
    // which MASKED the proof clause it also tripped. A reader who
    // repaired the refusal they saw would fix the commitment and be met
    // at once by the range proof, with neither refusal saying a second
    // one was waiting.
    //
    // The region is now decided before the clauses that guard it, so the
    // coin is judged by its own region's rules and never meets the
    // explicit-only ones at all.
    let policy = "bb".repeat(32);
    let mut blinded = transaction(&policy);
    blinded.outputs[2] = sponsor_coin(&policy, &[0x51]);

    let regions = classify_funding_members(&blinded, protocol().as_str(), 2, Some(&[0x51]))
        .expect("the sponsor coin places");
    assert_eq!(
        regions[2],
        FundingRegion::NonProtocol(NonProtocolFundingRegion::SponsorReserve),
        "a declared sponsor coin is still being read as somebody else's member",
    );
    // Its neighbours are untouched: the declaration moves ONE member.
    assert_eq!(
        regions[3],
        FundingRegion::NonProtocol(NonProtocolFundingRegion::PolicyFee)
    );

    // The declaration is what does it, and nothing about the bytes. The
    // SAME transaction with no program declared is read as policy change
    // and refused by that region's rule — which is why the program is
    // passed in rather than inferred from the value form.
    assert_eq!(
        classify_funding_members(&blinded, protocol().as_str(), 2, None).expect_err("it refuses"),
        RegionClassificationRefusal::NonProtocolMemberNotExplicit { index: 2 },
    );

    // And the region says which region it is, in words.
    assert_eq!(
        NonProtocolFundingRegion::SponsorReserve.to_string(),
        "the sponsor's reserve-asset coin",
    );
    assert!(!NonProtocolFundingRegion::SponsorReserve.requires_an_explicit_value());
    assert!(NonProtocolFundingRegion::PolicyChange.requires_an_explicit_value());
    assert!(NonProtocolFundingRegion::PolicyFee.requires_an_explicit_value());
}

#[test]
fn neither_explicit_only_refusal_stands_in_front_of_the_other() {
    // The masking, shown to be gone at the two members that still carry
    // the explicit-only rules. Each mutant breaks exactly ONE rule, so
    // each refusal is attributable to its own cause and neither is
    // reachable only by repairing the other first.
    let policy = "bb".repeat(32);

    // A commitment where the region requires an explicit value, with NO
    // proof beside it — so the proof clause has nothing to object to and
    // the refusal names the commitment alone.
    let mut committed = transaction(&policy);
    committed.outputs[2].value = DecodedValueField::Commitment(commitment(8));
    assert_eq!(
        classify_funding_members(&committed, protocol().as_str(), 2, None).expect_err("it refuses"),
        RegionClassificationRefusal::NonProtocolMemberNotExplicit { index: 2 },
    );

    // A proof beside a value that stays EXPLICIT — so the explicitness
    // clause has nothing to object to and the refusal names the proof
    // alone.
    let mut proved = transaction(&policy);
    proved.outputs[2].rangeproof = vec![0x01];
    assert_eq!(
        classify_funding_members(&proved, protocol().as_str(), 2, None).expect_err("it refuses"),
        RegionClassificationRefusal::NonProtocolMemberCarriesProof { index: 2 },
    );
}

#[test]
fn the_sponsor_region_refuses_each_of_its_four_rules_separately() {
    // Mutants first, one rule each. A sponsor coin's region admits what
    // the explicit-only regions forbid, which is exactly why its own
    // rules have to be checked rather than assumed: an admitted region
    // with no rules would let anything through that carried the right
    // program.
    let policy = "bb".repeat(32);
    let declared: Option<&[u8]> = Some(&[0x51]);
    let subject = |mutate: &dyn Fn(&mut DecodedFundingOutput)| {
        let mut transaction = transaction(&policy);
        let mut member = sponsor_coin(&policy, &[0x51]);
        mutate(&mut member);
        transaction.outputs[2] = member;
        transaction
    };

    // An explicit value is the degeneracy the region exists to refuse:
    // the number §15.2 asks to be private, written where anyone reads
    // it.
    assert_eq!(
        classify_funding_members(
            &subject(&|member| member.value = DecodedValueField::Explicit(1_250)),
            protocol().as_str(),
            2,
            declared,
        )
        .expect_err("it refuses"),
        RegionClassificationRefusal::SponsorMemberNotCommitted { index: 2 },
    );

    // A committed value with no range proof is refused by the target
    // itself, so it is refused here rather than carried to one.
    assert_eq!(
        classify_funding_members(
            &subject(&|member| member.rangeproof = Vec::new()),
            protocol().as_str(),
            2,
            declared,
        )
        .expect_err("it refuses"),
        RegionClassificationRefusal::SponsorMemberProofAbsent { index: 2 },
    );

    // A committed asset is the one thing this region may not hide: the
    // isolation fragment introspects it, and an introspection reads an
    // explicit field.
    assert_eq!(
        classify_funding_members(
            &subject(&|member| member.asset = DecodedAssetField::Commitment(commitment(10))),
            protocol().as_str(),
            2,
            declared,
        )
        .expect_err("it refuses"),
        RegionClassificationRefusal::SponsorMemberAssetNotExplicit { index: 2 },
    );

    // And a surjection proof beside an explicit asset, which the target
    // requires to be empty exactly because the asset is not committed.
    assert_eq!(
        classify_funding_members(
            &subject(&|member| member.surjection_proof = vec![0x03]),
            protocol().as_str(),
            2,
            declared,
        )
        .expect_err("it refuses"),
        RegionClassificationRefusal::SponsorMemberCarriesSurjectionProof { index: 2 },
    );
}

#[test]
fn a_protocol_position_must_carry_the_asset_and_a_committed_value() {
    let policy = "bb".repeat(32);
    let mut wrong_asset = transaction(&policy);
    wrong_asset.outputs[1] = protocol_output(&policy, &[0x52]);
    assert_eq!(
        classify_funding_members(&wrong_asset, protocol().as_str(), 2, None)
            .expect_err("it refuses"),
        RegionClassificationRefusal::ProtocolAssetAbsent { position: 1 }
    );
    let mut explicit = transaction(&policy);
    explicit.outputs[0].value = DecodedValueField::Explicit(7);
    assert_eq!(
        classify_funding_members(&explicit, protocol().as_str(), 2, None).expect_err("it refuses"),
        RegionClassificationRefusal::ProtocolValueNotCommitted { position: 0 }
    );
    let mut short = transaction(&policy);
    short.outputs.truncate(1);
    assert_eq!(
        classify_funding_members(&short, protocol().as_str(), 2, None).expect_err("it refuses"),
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
