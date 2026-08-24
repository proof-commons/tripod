//! The fixture registry, its grammar, its digest, and its derivation.
//!
//! # Convention
//!
//! These are running measurements and property checks over first-party
//! arithmetic. Nothing here reaches a target, nothing here is a verdict,
//! and every value under test is public disposable test material.

use std::collections::BTreeSet;

use num_bigint::BigUint;

use target_elements::ReproducibilityContract;

use crate::commitment_oracle::commitment::read_scalar;
use crate::commitment_oracle::curve::group_order;
use crate::confidential_fixture::{
    ConfidentialFixtureManifest, ConfidentialFixtureOutput, ConfidentialFixtureRegistry,
    DerivationRole, FixtureDerivationProfile, FixtureDiagnosticKind, FixtureOpenings,
    FixtureOutputRole, HandleGrammarDefect, MAX_PARITY_COUNTER, PublicDisposableTestMaterial,
    RegistrationRefusal, check_handle_grammar, predecessor_handle,
};
use crate::confidential_funding::FixtureResolutionRefused;
use crate::protocol::{
    ConfidentialFixtureDigest, ConfidentialFixtureHandle, ConfidentialFundingProfiles,
    FundingCustodyProfile, FundingMaterializerProfile, FundingRepresentationProfile,
};

/// A published disposable asset identity, in the order the target
/// commits to it in.
const ASSET: [u8; 32] = [0x3b; 32];

/// The profiles every manifest here selects.
fn profiles(contract: ReproducibilityContract) -> ConfidentialFundingProfiles {
    ConfidentialFundingProfiles {
        representation: FundingRepresentationProfile::ExplicitAssetConfidentialValue,
        custody: FundingCustodyProfile::CentralPublicFixtures,
        materializer: FundingMaterializerProfile::GuideCtfDeterministicV1,
        reproducibility_contract: contract,
    }
}

/// The predecessor manifest, as this wave registers it.
fn manifest() -> ConfidentialFixtureManifest {
    ConfidentialFixtureManifest {
        handle: predecessor_handle(),
        material_class: PublicDisposableTestMaterial::EXPECTED,
        derivation_profile: FixtureDerivationProfile::GuideCtfV1,
        profiles: profiles(ReproducibilityContract::ByteIdentity),
        retry_limit: MAX_PARITY_COUNTER,
        explicit_asset: ASSET,
        input_blinder_sum: [0_u8; 32],
        outputs: vec![
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Primary,
                semantic_amount: 700_000_000,
                output_program: vec![0x51],
            },
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Balancing,
                semantic_amount: 300_000_000,
                output_program: vec![0x51, 0x75, 0x51],
            },
        ],
    }
}

/// One frozen registry holding the predecessor.
fn frozen() -> crate::confidential_fixture::FrozenConfidentialFixtureRegistry {
    let mut registry = ConfidentialFixtureRegistry::new();
    registry
        .register(manifest())
        .expect("the manifest registers");
    registry.freeze()
}

#[test]
fn the_grammar_admits_the_slice_handle_and_refuses_the_rest() {
    check_handle_grammar(&predecessor_handle()).expect("the slice's own handle is admitted");
    let refused = |spelling: &str| {
        check_handle_grammar(&ConfidentialFixtureHandle::new(spelling.to_owned()))
            .expect_err("the grammar refuses it")
    };
    assert_eq!(
        refused("predecessor-dual-parity"),
        HandleGrammarDefect::PrefixAbsent
    );
    assert_eq!(
        refused("ctf-v1/ab"),
        HandleGrammarDefect::CaseNameTooShort { found: 2 }
    );
    assert_eq!(
        refused(&format!("ctf-v1/{}", "a".repeat(64))),
        HandleGrammarDefect::CaseNameTooLong { found: 64 }
    );
    // The one digit in the whole spelling belongs to the prefix, which
    // is what lets a later grammar be a different prefix rather than a
    // reinterpretation of this one.
    assert_eq!(
        refused("ctf-v1/case1"),
        HandleGrammarDefect::ByteNotAdmitted { position: 4 }
    );
    assert_eq!(refused("ctf-v1/-case"), HandleGrammarDefect::HyphenAtStart);
    assert_eq!(refused("ctf-v1/case-"), HandleGrammarDefect::HyphenAtEnd);
    assert_eq!(
        refused("ctf-v1/ca--se"),
        HandleGrammarDefect::AdjacentHyphens { position: 3 }
    );
    assert_eq!(
        refused("ctf-v1/CASE"),
        HandleGrammarDefect::ByteNotAdmitted { position: 0 }
    );
}

#[test]
fn the_parity_search_finds_the_admitted_pair_in_fixed_order() {
    let registry = frozen();
    let handle = predecessor_handle();
    let digest = *registry
        .registered_digest(&handle)
        .expect("the registry holds its own digest");
    let resolved = registry.resolve(&handle, &digest).expect("it resolves");
    let commitments = resolved
        .value_commitments()
        .expect("byte identity carries openings");
    let admitted = target_elements::reviewed_confidential_review_facts().value();
    let (low, high) = admitted.committed_prefixes();
    // The order is the fixture's and never a retry's: one square y then
    // one non-square, read from the reviewed target contract rather than
    // from a literal written here twice.
    assert_eq!(commitments[0][0], low);
    assert_eq!(commitments[1][0], high);
    assert!(admitted.admits_prefix(commitments[0][0]));
    assert!(admitted.admits_prefix(commitments[1][0]));
}

#[test]
fn the_two_value_blinders_are_ordered_additive_inverses() {
    let registry = frozen();
    let handle = predecessor_handle();
    let digest = *registry.registered_digest(&handle).expect("the digest");
    let resolved = registry.resolve(&handle, &digest).expect("it resolves");
    let FixtureOpenings::Derived { openings, .. } = resolved.openings() else {
        panic!("byte identity derives its openings");
    };
    let first = read_scalar(&openings[0].value_blinder).expect("a scalar");
    let second = read_scalar(&openings[1].value_blinder).expect("a scalar");
    assert_ne!(first, BigUint::from(0_u32));
    assert_ne!(second, BigUint::from(0_u32));
    // Their sum is the input blinder sum, which for this predecessor is
    // zero — so the two are ordered additive inverses as an instance of
    // the general balancing rule rather than as a special case.
    let order = group_order();
    let sum = (&first + &second) % order;
    assert_eq!(sum, BigUint::from(0_u32));
}

#[test]
fn derivation_is_role_separated_and_case_separated() {
    let registry = frozen();
    let handle = predecessor_handle();
    let digest = *registry.registered_digest(&handle).expect("the digest");
    let resolved = registry.resolve(&handle, &digest).expect("it resolves");
    let FixtureOpenings::Derived { openings, .. } = resolved.openings() else {
        panic!("byte identity derives its openings");
    };
    // Role separation: no role's value stands in for another's, and no
    // output's stands in for another output's.
    let mut seen: BTreeSet<[u8; 32]> = BTreeSet::new();
    for opening in openings {
        assert!(seen.insert(opening.value_blinder));
        assert!(seen.insert(opening.nonce_input));
        assert!(seen.insert(opening.rangeproof_seed));
    }
    assert_eq!(seen.len(), openings.len() * DerivationRole::ALL.len());

    // Case separation: a second case with the same shape and a different
    // handle derives nothing in common with the first.
    let mut other = ConfidentialFixtureRegistry::new();
    let mut second = manifest();
    second.handle = ConfidentialFixtureHandle::new("ctf-v1/predecessor-other-case".to_owned());
    other.register(second).expect("the second case registers");
    let other = other.freeze();
    let other_handle = ConfidentialFixtureHandle::new("ctf-v1/predecessor-other-case".to_owned());
    let other_digest = *other.registered_digest(&other_handle).expect("the digest");
    let other_resolved = other
        .resolve(&other_handle, &other_digest)
        .expect("it resolves");
    let FixtureOpenings::Derived {
        openings: theirs, ..
    } = other_resolved.openings()
    else {
        panic!("byte identity derives its openings");
    };
    for opening in theirs {
        assert!(!seen.contains(&opening.value_blinder));
        assert!(!seen.contains(&opening.nonce_input));
        assert!(!seen.contains(&opening.rangeproof_seed));
    }
    // And the digest moved with the case, which is what a drift digest
    // is for.
    assert_ne!(digest, other_digest);
}

#[test]
fn an_unknown_handle_and_a_drifted_digest_both_refuse() {
    let registry = frozen();
    let handle = predecessor_handle();
    let digest = *registry.registered_digest(&handle).expect("the digest");
    let stranger = ConfidentialFixtureHandle::new("ctf-v1/never-registered".to_owned());
    assert_eq!(
        registry.resolve(&stranger, &digest).expect_err("unknown"),
        FixtureResolutionRefused::UnknownHandle
    );
    assert_eq!(
        registry
            .resolve(&handle, &ConfidentialFixtureDigest::new([0_u8; 32]))
            .expect_err("drifted"),
        FixtureResolutionRefused::DigestMismatch
    );
    // A diagnostic may name the handle, the digest, and the kind, and
    // there is no member an amount or an opening could be written into.
    let diagnostic = registry.diagnose(&stranger, FixtureResolutionRefused::UnknownHandle);
    assert_eq!(diagnostic.kind(), FixtureDiagnosticKind::HandleUnknown);
    assert_eq!(diagnostic.handle(), &stranger);
    assert!(diagnostic.digest().is_none());
    assert!(diagnostic.role().is_none());
    assert!(diagnostic.counter().is_none());
}

#[test]
fn an_equal_re_registration_is_refused_as_a_duplicate() {
    let mut registry = ConfidentialFixtureRegistry::new();
    registry.register(manifest()).expect("the first registers");
    // Refused even though the two manifests are equal: a registry that
    // silently accepted an equal re-registration would accept an unequal
    // one on the day the two drifted.
    assert_eq!(
        registry
            .register(manifest())
            .expect_err("the second refuses"),
        RegistrationRefusal::DuplicateFixtureHandle
    );
}

#[test]
fn registration_refuses_every_clause_it_states() {
    let refused = |edit: &dyn Fn(&mut ConfidentialFixtureManifest)| {
        let mut subject = manifest();
        edit(&mut subject);
        ConfidentialFixtureRegistry::new()
            .register(subject)
            .expect_err("the clause refuses")
    };
    assert_eq!(
        refused(&|subject| subject.handle = ConfidentialFixtureHandle::new("bad".to_owned())),
        RegistrationRefusal::HandleGrammar {
            defect: HandleGrammarDefect::PrefixAbsent
        }
    );
    assert_eq!(
        refused(&|subject| subject.outputs.truncate(1)),
        RegistrationRefusal::OutputSetTooSmall { found: 1 }
    );
    assert_eq!(
        refused(&|subject| subject.outputs[1].role = FixtureOutputRole::Primary),
        RegistrationRefusal::BalancingRoleNotUnique { found: 0 }
    );
    assert_eq!(
        refused(&|subject| subject.outputs[0].semantic_amount = 0),
        RegistrationRefusal::AmountNotPositive { output: 0 }
    );
    assert_eq!(
        refused(&|subject| subject.outputs[0].semantic_amount = u64::MAX),
        RegistrationRefusal::AmountNotSemantic { output: 0 }
    );
    assert_eq!(
        refused(&|subject| subject.outputs[1].output_program.clear()),
        RegistrationRefusal::OutputProgramEmpty { output: 1 }
    );
    assert_eq!(
        refused(&|subject| subject.retry_limit = MAX_PARITY_COUNTER + 1),
        RegistrationRefusal::RetryLimitAboveBound {
            stated: MAX_PARITY_COUNTER + 1
        }
    );
    assert_eq!(
        refused(&|subject| subject.input_blinder_sum = [0xff_u8; 32]),
        RegistrationRefusal::InputBlinderSumInvalid
    );
}

#[test]
fn a_recorded_randomness_case_carries_no_openings_and_a_different_digest() {
    let mut byte_identity = ConfidentialFixtureRegistry::new();
    byte_identity.register(manifest()).expect("it registers");
    let byte_identity = byte_identity.freeze();
    let handle = predecessor_handle();
    let reference = *byte_identity
        .registered_digest(&handle)
        .expect("the digest");

    let mut recorded = ConfidentialFixtureRegistry::new();
    let mut subject = manifest();
    subject.profiles = profiles(ReproducibilityContract::RecordedRandomness);
    recorded.register(subject).expect("it registers");
    let recorded = recorded.freeze();
    let other = *recorded.registered_digest(&handle).expect("the digest");
    let resolved = recorded.resolve(&handle, &other).expect("it resolves");

    // The openings do not exist until the run produces them, and the
    // digest binds the semantic transcript alone. The contract tag sits
    // inside that transcript, which is what makes the two impossible to
    // confuse.
    assert!(matches!(resolved.openings(), FixtureOpenings::RunProduced));
    assert!(resolved.value_commitments().is_none());
    assert_ne!(reference, other);
}

#[test]
fn the_material_class_is_the_one_the_construction_side_states() {
    // The registry's class and the construction side's non-claim are
    // held equal by this test rather than by intent.
    assert_eq!(
        PublicDisposableTestMaterial::EXPECTED,
        PublicDisposableTestMaterial::DisposableTestNetworkMaterial
    );
    // The construction side states the same fact in its own
    // vocabulary, and the two are held equal here rather than by intent.
    assert!(
        transaction::live_private::PrivateConstructionNonClaim::ALL
            .contains(&transaction::live_private::PrivateConstructionNonClaim::NoOpeningIsSecret)
    );
}
