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
    DerivationRole, DerivationSource, DerivedOpening, FIXTURE_DIGEST_TAG, FixtureDerivationProfile,
    FixtureDerivationRefusal, FixtureDiagnosticKind, FixtureOpenings, FixtureOutputRole,
    HandleGrammarDefect, MAX_PARITY_COUNTER, PublicDisposableTestMaterial, RegistrationRefusal,
    TaggedHashDerivation, check_handle_grammar, digest_transcript, predecessor_handle,
};
use crate::confidential_funding::FixtureResolutionRefused;
use crate::constructor::tagged::tagged_hash;
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

/// The openings of the outputs that carry one.
///
/// Openings are optional per output because an explicit output has none.
/// Every fixture in this file is all-blinded, so this is every output of
/// it; the helper exists so that a test about a blinder reads a blinder
/// rather than an option of one, and so that an unexpectedly absent
/// opening fails loudly here instead of being skipped quietly.
fn committed(openings: &[Option<DerivedOpening>]) -> Vec<&DerivedOpening> {
    openings
        .iter()
        .map(|opening| {
            opening
                .as_ref()
                .expect("a blinded output carries an opening")
        })
        .collect()
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
    let openings = committed(openings);
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
    let openings = committed(openings);
    // Role separation: no role's value stands in for another's, and no
    // output's stands in for another output's.
    let mut seen: BTreeSet<[u8; 32]> = BTreeSet::new();
    for opening in &openings {
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
    let theirs = committed(theirs);
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

fn registered_digest(manifest: ConfidentialFixtureManifest) -> ConfidentialFixtureDigest {
    let handle = manifest.handle.clone();
    let mut registry = ConfidentialFixtureRegistry::new();
    registry.register(manifest).expect("the manifest registers");
    *registry
        .freeze()
        .registered_digest(&handle)
        .expect("the registry holds the digest")
}

fn recorded_manifest_for_opening_role(
    role: FixtureOutputRole,
) -> (ConfidentialFixtureManifest, usize) {
    let (mut subject, output) = match role {
        FixtureOutputRole::Primary => (manifest(), 0),
        FixtureOutputRole::Balancing => (manifest(), 1),
        FixtureOutputRole::SoleBalancing => (
            sole_manifest("ctf-v1/digest-sole-balancing", NON_CANCELING_SUM),
            0,
        ),
        FixtureOutputRole::SponsorChange { asset } => (
            sponsored_change_manifest(
                "ctf-v1/digest-sponsor-change",
                FixtureOutputRole::SponsorChange { asset },
            ),
            1,
        ),
        FixtureOutputRole::BalancingSponsorChange { asset } => (
            sole_blinded_change_manifest(
                "ctf-v1/digest-balancing-sponsor-change",
                FixtureOutputRole::BalancingSponsorChange { asset },
                NON_CANCELING_SUM,
            ),
            2,
        ),
        FixtureOutputRole::Fee
        | FixtureOutputRole::ExplicitDestination
        | FixtureOutputRole::ExplicitSponsorChange { .. } => {
            panic!("the regression covers only opening-bearing roles")
        }
    };
    subject.profiles = profiles(ReproducibilityContract::RecordedRandomness);
    (subject, output)
}

#[test]
fn recorded_randomness_v2_binds_amounts_for_every_opening_bearing_role() {
    let cases = [
        ("primary", FixtureOutputRole::Primary),
        ("balancing", FixtureOutputRole::Balancing),
        ("sole-balancing", FixtureOutputRole::SoleBalancing),
        (
            "sponsor-change",
            FixtureOutputRole::SponsorChange {
                asset: RESERVE_ASSET,
            },
        ),
        (
            "balancing-sponsor-change",
            FixtureOutputRole::BalancingSponsorChange {
                asset: RESERVE_ASSET,
            },
        ),
    ];
    let mut collisions = Vec::new();
    for (name, role) in cases {
        let (baseline, output) = recorded_manifest_for_opening_role(role);
        let mut mutated = baseline.clone();
        mutated.outputs[output].semantic_amount += 1;
        if registered_digest(baseline) == registered_digest(mutated) {
            collisions.push(name);
        }
    }
    assert!(
        collisions.is_empty(),
        "semantic amount drift collided for opening-bearing roles: {collisions:?}",
    );
}

#[test]
fn v2_binds_a_value_conserving_amount_mutation_under_both_contracts() {
    let baseline = manifest();
    let mut mutated = baseline.clone();
    mutated.outputs[0].semantic_amount -= 1;
    mutated.outputs[1].semantic_amount += 1;
    for contract in ReproducibilityContract::ALL {
        assert_ne!(
            digest_under_contract(&baseline, contract),
            digest_under_contract(&mutated, contract),
        );
    }
}

const PYTHON_EXECUTOR_SOURCE: &str =
    include_str!("../../../../scripts/elements-native-executor.py");
const RUST_FIXTURE_SOURCE: &str = include_str!("../confidential_fixture.rs");

fn hex_of(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}

fn python_digest_golden(name: &str) -> String {
    let assignment = format!("{name} = (\n");
    let (_, after_assignment) = PYTHON_EXECUTOR_SOURCE
        .split_once(&assignment)
        .expect("the Python source carries the named golden");
    let (body, _) = after_assignment
        .split_once("\n)")
        .expect("the Python golden closes its tuple");
    body.lines()
        .map(str::trim)
        .map(|line| {
            line.strip_prefix('"')
                .and_then(|line| line.strip_suffix('"'))
                .expect("each Python golden line is one string literal")
        })
        .collect()
}

fn golden_manifest(contract: ReproducibilityContract) -> ConfidentialFixtureManifest {
    ConfidentialFixtureManifest {
        handle: ConfidentialFixtureHandle::new("ctf-v1/vtwo-golden".to_owned()),
        material_class: PublicDisposableTestMaterial::EXPECTED,
        derivation_profile: FixtureDerivationProfile::GuideCtfV1,
        profiles: profiles(contract),
        retry_limit: 5,
        explicit_asset: [0x11; 32],
        input_blinder_sum: [0x01; 32],
        outputs: vec![
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Primary,
                semantic_amount: 1,
                output_program: vec![0x51],
            },
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Balancing,
                semantic_amount: 2,
                output_program: vec![0x52, 0x53],
            },
        ],
    }
}

fn golden_opening(value: u8, nonce: u8, seed: u8, prefix: u8) -> DerivedOpening {
    let mut value_commitment = [0_u8; 33];
    value_commitment[0] = prefix;
    DerivedOpening {
        value_blinder: [value; 32],
        nonce_input: [nonce; 32],
        rangeproof_seed: [seed; 32],
        value_commitment,
    }
}

fn golden_openings() -> FixtureOpenings {
    FixtureOpenings::Derived {
        parity_counter: 7,
        openings: vec![
            Some(golden_opening(0x21, 0x31, 0x41, 0x08)),
            Some(golden_opening(0x22, 0x32, 0x42, 0x09)),
        ],
    }
}

fn assert_python_golden(
    contract: ReproducibilityContract,
    openings: &FixtureOpenings,
    transcript_name: &str,
    digest_name: &str,
) {
    let transcript = digest_transcript(&golden_manifest(contract), openings);
    assert_eq!(hex_of(&transcript), python_digest_golden(transcript_name));
    assert_eq!(
        hex_of(&tagged_hash(FIXTURE_DIGEST_TAG, &transcript)),
        python_digest_golden(digest_name),
    );
}

#[test]
fn rust_and_python_share_v2_goldens_for_both_contracts() {
    assert_python_golden(
        ReproducibilityContract::ByteIdentity,
        &golden_openings(),
        "CONFIDENTIAL_DIGEST_V2_BYTE_IDENTITY_TRANSCRIPT_HEX",
        "CONFIDENTIAL_DIGEST_V2_BYTE_IDENTITY_DIGEST_HEX",
    );
    assert_python_golden(
        ReproducibilityContract::RecordedRandomness,
        &FixtureOpenings::RunProduced,
        "CONFIDENTIAL_DIGEST_V2_RECORDED_RANDOMNESS_TRANSCRIPT_HEX",
        "CONFIDENTIAL_DIGEST_V2_RECORDED_RANDOMNESS_DIGEST_HEX",
    );
}

type FixtureMutation = fn(&mut ConfidentialFixtureManifest);

fn digest_under_contract(
    manifest: &ConfidentialFixtureManifest,
    contract: ReproducibilityContract,
) -> ConfidentialFixtureDigest {
    let mut subject = manifest.clone();
    subject.profiles = profiles(contract);
    registered_digest(subject)
}

#[test]
fn semantic_mutations_change_the_digest_under_both_contracts() {
    let mutations: [(&str, FixtureMutation); 5] = [
        ("amount", |subject| subject.outputs[0].semantic_amount += 1),
        ("role", |subject| {
            subject.outputs[0].role = FixtureOutputRole::Balancing;
            subject.outputs[1].role = FixtureOutputRole::Primary;
        }),
        ("asset", |subject| subject.explicit_asset[0] ^= 0x01),
        ("program", |subject| {
            subject.outputs[0].output_program.push(0x00);
        }),
        ("order", |subject| subject.outputs.swap(0, 1)),
    ];
    let baseline = manifest();
    for contract in ReproducibilityContract::ALL {
        let baseline_digest = digest_under_contract(&baseline, contract);
        for (name, mutate) in mutations {
            let mut mutated = baseline.clone();
            mutate(&mut mutated);
            assert_ne!(
                baseline_digest,
                digest_under_contract(&mutated, contract),
                "{contract} failed to bind the {name} mutation",
            );
        }
    }
}

#[test]
fn the_reproducibility_profile_changes_the_digest_in_both_directions() {
    let subject = manifest();
    let byte_identity = digest_under_contract(&subject, ReproducibilityContract::ByteIdentity);
    let recorded = digest_under_contract(&subject, ReproducibilityContract::RecordedRandomness);
    assert_ne!(byte_identity, recorded);
    assert_ne!(recorded, byte_identity);
}

#[test]
fn constructor_only_input_sum_changes_openings_but_not_recorded_randomness_digest() {
    let baseline = sole_manifest("ctf-v1/digest-input-sum-control", NON_CANCELING_SUM);
    let mut mutated = baseline.clone();
    mutated.input_blinder_sum[31] += 1;
    assert_ne!(
        digest_under_contract(&baseline, ReproducibilityContract::ByteIdentity),
        digest_under_contract(&mutated, ReproducibilityContract::ByteIdentity),
    );
    assert_eq!(
        digest_under_contract(&baseline, ReproducibilityContract::RecordedRandomness),
        digest_under_contract(&mutated, ReproducibilityContract::RecordedRandomness),
    );
}

#[test]
fn v2_is_the_only_live_digest_emitter_on_both_sides() {
    const V2_TAG: &str = "tripod/guide-ctf/fixture-digest/v2";
    assert_eq!(FIXTURE_DIGEST_TAG, V2_TAG);
    assert!(PYTHON_EXECUTOR_SOURCE.contains(&format!("CONFIDENTIAL_DIGEST_TAG = b\"{V2_TAG}\"")));

    let deleted_tag = ["tripod/guide-ctf/fixture-digest/", "v1"].concat();
    assert!(!RUST_FIXTURE_SOURCE.contains(&deleted_tag));
    assert!(!PYTHON_EXECUTOR_SOURCE.contains(&deleted_tag));

    for deleted_emitter in [
        "digest_transcript_v1",
        "fixture_digest_v1",
        "confidential_digest_transcript_v1",
    ] {
        assert!(!RUST_FIXTURE_SOURCE.contains(deleted_emitter));
        assert!(!PYTHON_EXECUTOR_SOURCE.contains(deleted_emitter));
    }
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

// --- The derivation source seam ----------------------------------------
//
// Six of `FixtureDerivationRefusal`'s variants are refusals about derived
// material, and until the seam existed no fixture could reach one: the
// tagged hash answers what it answers. The tests below reach two of them
// and record honestly which the seam does not reach and why.

/// A source answering one constant to every preimage.
///
/// The whole point of a constant is that it makes the two outputs' value
/// blinders equal, which is a relation no hash would produce and which
/// the balance arithmetic has a definite opinion about.
struct ConstantDerivation([u8; 32]);

impl DerivationSource for ConstantDerivation {
    fn derive(&self, _tag: &str, _preimage: &[u8]) -> [u8; 32] {
        self.0
    }
}

/// The scalar the constant source answers with.
///
/// Far below the group order and plainly nonzero, so that it is admitted
/// as a scalar and the refusal under test is the one about the SOLVE
/// rather than one about the derivation.
const CONSTANT_SCALAR: [u8; 32] = [0x01; 32];

/// The seam changed nothing the deterministic source produces.
///
/// The first thing to check about a seam, and the one a reviewer would
/// ask for: the public entry point and the deterministic source produce
/// the same registration, digest included. A digest is what every
/// recorded funding record binds to, so a seam that moved one would have
/// invalidated every fixture already published.
#[test]
fn the_seam_leaves_the_deterministic_registration_byte_identical() {
    let mut through_public = ConfidentialFixtureRegistry::new();
    through_public
        .register(manifest())
        .expect("the predecessor manifest registers");

    let mut through_seam = ConfidentialFixtureRegistry::new();
    through_seam
        .register_with_source(manifest(), &TaggedHashDerivation)
        .expect("and registers the same way through the seam");

    let public = through_public.freeze();
    let seam = through_seam.freeze();
    let handle = predecessor_handle();

    assert_eq!(
        public.registered_digest(&handle),
        seam.registered_digest(&handle),
        "the deterministic source is exactly what the public entry point selects",
    );
}

/// A degenerate solved balancing scalar is a typed refusal.
///
/// The solve is the input blinder sum minus the other outputs' sum. A
/// source answering one constant makes the only other output's blinder
/// that constant, so declaring the same constant as the input blinder sum
/// makes the solve land on zero exactly. The rule under test is that it
/// is refused rather than nudged: there is no arm here that adds one and
/// tries again.
#[test]
fn a_degenerate_balancing_scalar_is_a_typed_refusal() {
    let mut manifest = manifest();
    manifest.input_blinder_sum = CONSTANT_SCALAR;

    let refusal = ConfidentialFixtureRegistry::new()
        .register_with_source(manifest, &ConstantDerivation(CONSTANT_SCALAR))
        .expect_err("a solved zero is not a blinder");

    assert_eq!(
        refusal,
        RegistrationRefusal::Derivation {
            refusal: FixtureDerivationRefusal::DegenerateBalancingScalar,
        },
        "the refusal names the solve and not the search",
    );
}

/// A scalar search that never admits is a typed refusal, and it names its
/// role and its attempts.
///
/// A source answering zero derives nothing admissible at any counter, so
/// the bounded upward search runs to its bound and refuses. What the test
/// pins is the count: the search moves from zero to the bound inclusive
/// without wrapping and without skipping, which is `MAX_SCALAR_COUNTER`
/// plus one attempts and not one more.
#[test]
fn a_scalar_search_that_never_admits_refuses_at_its_bound() {
    let refusal = ConfidentialFixtureRegistry::new()
        .register_with_source(manifest(), &ConstantDerivation([0_u8; 32]))
        .expect_err("zero is not an admitted scalar at any counter");

    assert_eq!(
        refusal,
        RegistrationRefusal::Derivation {
            refusal: FixtureDerivationRefusal::ScalarSearchExhausted {
                role: DerivationRole::ValueBlinder,
                attempts: 256,
            },
        },
        "the first role searched, and one attempt per admitted counter",
    );
}

/// The seam does not reach an identity value commitment, and the reason
/// is a property of the arithmetic rather than a gap in the seam.
///
/// A commitment is the semantic amount on the asset generator plus the
/// blinder on the base point, and it lands on the group identity only
/// where the blinder is the discrete logarithm of the negated amount
/// term. Choosing bytes does not choose that: a source picks a blinder,
/// and no blinder anyone can write down is that one.
///
/// Reaching `IdentityValueCommitment` from here would therefore need a
/// seam on the commitment arithmetic itself, and that arithmetic is the
/// workspace's independence claim — a test that could replace it would
/// have made the claim checkable by substitution. So the refusal is
/// reached at the layer that genuinely injects its cryptography instead,
/// where the materializer's `InvalidCommitment` covers it, and this test
/// records that the constant source does NOT reach it rather than leaving
/// a reader to wonder whether anyone tried.
#[test]
fn the_constant_source_reaches_a_solve_refusal_and_not_an_identity_one() {
    let mut manifest = manifest();
    manifest.input_blinder_sum = CONSTANT_SCALAR;

    let refusal = ConfidentialFixtureRegistry::new()
        .register_with_source(manifest, &ConstantDerivation(CONSTANT_SCALAR))
        .expect_err("the constant source refuses");

    assert!(
        !matches!(
            refusal,
            RegistrationRefusal::Derivation {
                refusal: FixtureDerivationRefusal::IdentityValueCommitment { .. },
            },
        ),
        "no choice of derived bytes puts a commitment on the identity",
    );
}

/// A fixture wider than two outputs derives at all.
///
/// # What was wrong, and what it cost
///
/// The bounded parity search compared the target's admitted prefix pair
/// against the openings BY LENGTH, so a manifest of any width but two
/// matched no counter and exhausted the search after four thousand and
/// ninety-six attempts. Nothing in the target contract says a confidential
/// transaction has two outputs; the cardinality was this file's own
/// assumption, and it was the deepest layer of the absent multi-output
/// shape constructor — deeper than the manifest builder, which had always
/// accepted a wider output set and then could not derive it.
#[test]
fn a_fixture_wider_than_two_outputs_derives() {
    let mut wide = manifest();
    wide.handle = ConfidentialFixtureHandle::new("ctf-v1/three-output-probe".to_owned());
    wide.outputs = vec![
        ConfidentialFixtureOutput {
            role: FixtureOutputRole::Primary,
            semantic_amount: 400_000_000,
            output_program: vec![0x51],
        },
        ConfidentialFixtureOutput {
            role: FixtureOutputRole::Primary,
            semantic_amount: 200_000_000,
            output_program: vec![0x51, 0x75, 0x51],
        },
        ConfidentialFixtureOutput {
            role: FixtureOutputRole::Balancing,
            semantic_amount: 100_000_000,
            output_program: vec![0x52, 0x20, 0xaa],
        },
    ];
    let handle = wide.handle.clone();

    let mut registry = ConfidentialFixtureRegistry::new();
    registry
        .register(wide)
        .expect("a three-output fixture registers");
    let registry = registry.freeze();
    let digest = *registry.registered_digest(&handle).expect("the digest");
    let resolved = registry.resolve(&handle, &digest).expect("it resolves");
    let FixtureOpenings::Derived { openings, .. } = resolved.openings() else {
        panic!("byte identity derives its openings");
    };
    let openings = committed(openings);

    // One opening per output, and every commitment carries an admitted
    // prefix — which is the whole of what the target contract states about
    // a set of this width.
    assert_eq!(openings.len(), 3);
    for opening in openings {
        assert!(
            opening.value_commitment[0] == 0x08 || opening.value_commitment[0] == 0x09,
            "an output's commitment carries a prefix the target does not admit: {:#04x}",
            opening.value_commitment[0],
        );
    }
}

/// The two-output rule is exactly what it was.
///
/// The generalization above must not have moved the case the search was
/// built for. A two-output fixture is still held to the admitted pair in
/// FIXED ORDER — first output at the first prefix, second at the second —
/// which is what makes the dual-parity predecessor carry one of each
/// parity rather than one of them twice. If this loosened to the wider
/// rule, a predecessor whose outputs both carried the same parity would
/// derive, and the parity evidence would be about one form claimed twice.
#[test]
fn the_two_output_prefix_pair_is_still_required_in_fixed_order() {
    let registry = frozen();
    let handle = predecessor_handle();
    let digest = *registry.registered_digest(&handle).expect("the digest");
    let resolved = registry.resolve(&handle, &digest).expect("it resolves");
    let FixtureOpenings::Derived { openings, .. } = resolved.openings() else {
        panic!("byte identity derives its openings");
    };
    let openings = committed(openings);
    assert_eq!(openings.len(), 2);
    assert_eq!(
        openings[0].value_commitment[0], 0x08,
        "the first output carries the first admitted prefix",
    );
    assert_eq!(
        openings[1].value_commitment[0], 0x09,
        "the second output carries the second admitted prefix",
    );
}

// --- The single-output fully-solved balancing form ---------------------

/// One manifest of a single sole-balancing output.
///
/// The input blinder sum is the caller's, because it is the whole
/// question this form asks: the lone output's blinder IS that sum, so a
/// test that fixed it here could not reach the degenerate case.
fn sole_manifest(handle: &str, input_blinder_sum: [u8; 32]) -> ConfidentialFixtureManifest {
    ConfidentialFixtureManifest {
        handle: ConfidentialFixtureHandle::new(handle.to_owned()),
        material_class: PublicDisposableTestMaterial::EXPECTED,
        derivation_profile: FixtureDerivationProfile::GuideCtfV1,
        profiles: profiles(ReproducibilityContract::ByteIdentity),
        retry_limit: MAX_PARITY_COUNTER,
        explicit_asset: ASSET,
        input_blinder_sum,
        outputs: vec![ConfidentialFixtureOutput {
            role: FixtureOutputRole::SoleBalancing,
            semantic_amount: 1_000_000_000,
            output_program: vec![0x51],
        }],
    }
}

/// A nonzero input blinder sum, standing in for a predecessor whose
/// consumed blinders do not cancel.
///
/// Public disposable test material under ADR-015. It is small and
/// obviously below the group order, which is all this form asks of it.
const NON_CANCELING_SUM: [u8; 32] = {
    let mut bytes = [0_u8; 32];
    bytes[31] = 0x2a;
    bytes
};

#[test]
fn the_single_output_form_registers_and_takes_the_input_blinder_sum_verbatim() {
    let mut registry = ConfidentialFixtureRegistry::new();
    registry
        .register(sole_manifest("ctf-v1/sole-form", NON_CANCELING_SUM))
        .expect("the declared single-output form registers");
    let frozen = registry.freeze();
    let handle = ConfidentialFixtureHandle::new("ctf-v1/sole-form".to_owned());
    let digest = *frozen
        .registered_digest(&handle)
        .expect("the registry holds its own digest");
    let resolved = frozen.resolve(&handle, &digest).expect("it resolves");

    let FixtureOpenings::Derived {
        openings,
        parity_counter,
    } = resolved.openings()
    else {
        panic!("byte identity derives openings");
    };
    let openings = committed(openings);
    assert_eq!(openings.len(), 1, "the form has exactly one output");

    // The whole claim of the form: the blinder is not derived and not
    // searched for. It is the input blinder sum, unchanged.
    assert_eq!(
        openings[0].value_blinder, NON_CANCELING_SUM,
        "the lone output's blinder is FORCED to the input blinder sum",
    );

    // And the search that would have chosen it has nothing to choose, so
    // it settles at the first counter. This is asserted rather than
    // described because a bounded search that cannot fail must not be
    // reported as one that discriminated.
    assert_eq!(
        *parity_counter, 0,
        "the parity search degenerates to a well-formedness check",
    );

    // Whichever prefix the forced blinder yields is the one it carries.
    // Both are admitted and neither was selected.
    let admitted = target_elements::reviewed_confidential_review_facts().value();
    assert!(
        admitted.admits_prefix(openings[0].value_commitment[0]),
        "the commitment's parity is whatever the forced blinder yields",
    );
}

#[test]
fn a_canceling_predecessor_is_refused_rather_than_built_hiding_nothing() {
    // THE DEGENERACY, held as a refusal.
    //
    // A zero input blinder sum is what merging the two halves of an
    // inverse pair produces: the halves cancel by construction, which is
    // what makes them an inverse pair. The forced blinder would then be
    // zero and the commitment exactly the value times the value
    // generator — a point anyone recomputes from a guessed amount,
    // carrying a blinded output's form and none of its hiding.
    //
    // The registry does not build it and does not annotate it. It
    // refuses.
    let refusal = ConfidentialFixtureRegistry::new()
        .register(sole_manifest("ctf-v1/sole-canceling", [0_u8; 32]))
        .expect_err("a canceling predecessor is refused");
    assert_eq!(
        refusal,
        RegistrationRefusal::Derivation {
            refusal: FixtureDerivationRefusal::DegenerateBalancingScalar
        },
        "the zero solved blinder is refused outright and never nudged",
    );
}

#[test]
fn a_lone_output_that_does_not_declare_the_form_still_meets_the_floor() {
    // The removal is a narrowing and not a relaxation. Nothing that was
    // refused before registers now unless it says which form it means,
    // and the refusal it draws is the same one with the same count.
    let refused = |role: FixtureOutputRole| {
        let mut subject = sole_manifest("ctf-v1/sole-undeclared", NON_CANCELING_SUM);
        subject.outputs[0].role = role;
        ConfidentialFixtureRegistry::new()
            .register(subject)
            .expect_err("an undeclared lone output is refused")
    };
    assert_eq!(
        refused(FixtureOutputRole::Balancing),
        RegistrationRefusal::OutputSetTooSmall { found: 1 },
        "a lone output asking to be solved from others that are not there",
    );
    assert_eq!(
        refused(FixtureOutputRole::Primary),
        RegistrationRefusal::OutputSetTooSmall { found: 1 },
    );
}

#[test]
fn the_sole_form_is_refused_beside_other_outputs() {
    // The form is the whole manifest. Read as a wider manifest's
    // balancing output it would let a caller reach the single-output
    // solve without the single-output shape.
    let mut subject = manifest();
    subject.outputs[1].role = FixtureOutputRole::SoleBalancing;
    assert_eq!(
        ConfidentialFixtureRegistry::new()
            .register(subject)
            .expect_err("the role is refused beside others"),
        RegistrationRefusal::SoleBalancingRoleNotAlone { found: 2 },
    );
}

#[test]
fn the_two_output_predecessor_derives_exactly_what_it_derived_before() {
    // The bit-for-bit clause of the removal, held against the values the
    // two-output case had before the single-output form existed rather
    // than against a fresh recomputation.
    let registry = frozen();
    let handle = predecessor_handle();
    let digest = *registry
        .registered_digest(&handle)
        .expect("the registry holds its own digest");
    let resolved = registry.resolve(&handle, &digest).expect("it resolves");
    let FixtureOpenings::Derived {
        openings,
        parity_counter,
    } = resolved.openings()
    else {
        panic!("byte identity derives openings");
    };
    let openings = committed(openings);

    // The dual-parity rule: the admitted pair in FIXED ORDER, which is
    // the discriminating power the search really does have and which the
    // widened rule left untouched.
    let admitted = target_elements::reviewed_confidential_review_facts().value();
    let (low, high) = admitted.committed_prefixes();
    assert_eq!(openings[0].value_commitment[0], low);
    assert_eq!(openings[1].value_commitment[0], high);

    // The two blinders are ordered additive inverses.
    let order = group_order();
    let first = read_scalar(&openings[0].value_blinder).expect("a scalar");
    let second = read_scalar(&openings[1].value_blinder).expect("a scalar");
    assert_eq!((first + second) % order, BigUint::from(0_u32));

    // And the derivation repeats, at the same counter, byte for byte.
    // The counter is not asserted to be any particular number here:
    // what the two-output case is owed is that it derives what it
    // derived, and the run-of-record digest check in the restart lane is
    // where that is held against values recorded before this form
    // existed.
    let mut again = ConfidentialFixtureRegistry::new();
    again.register(manifest()).expect("it registers again");
    let again = again.freeze();
    let repeated = again.resolve(&handle, &digest).expect("the same digest");
    assert_eq!(repeated.openings(), resolved.openings());
    let FixtureOpenings::Derived {
        parity_counter: repeated_counter,
        ..
    } = repeated.openings()
    else {
        panic!("byte identity derives openings");
    };
    assert_eq!(repeated_counter, parity_counter);
}

// --- The fee output role -----------------------------------------------

/// One manifest of a blinded output beside a fee output.
///
/// The blinded output BALANCES: it is the only output that can, the fee
/// contributing a zero blinder by the target's own definition of a fee.
fn fee_bearing_manifest(program: Vec<u8>) -> ConfidentialFixtureManifest {
    ConfidentialFixtureManifest {
        handle: ConfidentialFixtureHandle::new("ctf-v1/fee-bearing".to_owned()),
        material_class: PublicDisposableTestMaterial::EXPECTED,
        derivation_profile: FixtureDerivationProfile::GuideCtfV1,
        profiles: profiles(ReproducibilityContract::ByteIdentity),
        retry_limit: MAX_PARITY_COUNTER,
        explicit_asset: ASSET,
        input_blinder_sum: NON_CANCELING_SUM,
        outputs: vec![
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Balancing,
                semantic_amount: 900_000_000,
                output_program: vec![0x51],
            },
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Fee,
                semantic_amount: 100_000_000,
                output_program: program,
            },
        ],
    }
}

#[test]
fn a_fee_bearing_manifest_registers_with_the_fee_held_out_of_the_solve() {
    let mut registry = ConfidentialFixtureRegistry::new();
    registry
        .register(fee_bearing_manifest(Vec::new()))
        .expect("a fee-bearing manifest registers");
    let frozen = registry.freeze();
    let handle = ConfidentialFixtureHandle::new("ctf-v1/fee-bearing".to_owned());
    let digest = *frozen.registered_digest(&handle).expect("its own digest");
    let resolved = frozen.resolve(&handle, &digest).expect("it resolves");

    let FixtureOpenings::Derived { openings, .. } = resolved.openings() else {
        panic!("byte identity derives openings");
    };
    assert_eq!(openings.len(), 2, "one entry per output, never compacted");

    // The fee's opening is ABSENT rather than zero-filled. A record of
    // zeroes would read like an opening, and an explicit output has none.
    assert!(
        openings[1].is_none(),
        "an explicit output carries no opening at all",
    );

    // And the fee is held OUT of the solve at a zero blinder, so the
    // blinded output takes the whole input blinder sum — exactly what it
    // would take with no fee output present. That is the fee role's whole
    // arithmetic claim.
    let blinded = openings[0]
        .as_ref()
        .expect("the blinded output carries an opening");
    assert_eq!(
        blinded.value_blinder, NON_CANCELING_SUM,
        "the fee contributes a zero blinder, so the solve is unchanged by it",
    );

    // One commitment, not two: the commitments are the outputs that have
    // one, and a fee has none.
    assert_eq!(
        resolved
            .value_commitments()
            .expect("byte identity carries openings")
            .len(),
        1,
    );
}

#[test]
fn a_fee_output_carrying_a_program_is_refused() {
    // REQUIRED empty, not merely permitted. An empty scriptPubKey is the
    // fee's identity at the target, so a fee carrying a program is a fee
    // the target would not read as one — and a role that only tolerated
    // an empty program would let a manifest declare exactly that.
    assert_eq!(
        ConfidentialFixtureRegistry::new()
            .register(fee_bearing_manifest(vec![0x51]))
            .expect_err("a fee output with a program is refused"),
        RegistrationRefusal::FeeProgramNotEmpty { output: 1 },
    );
}

#[test]
fn a_fee_output_may_not_be_the_only_output_and_may_not_balance() {
    // The fee-only shape stays refused, and this is the register's one
    // consensus-IMPOSSIBLE shape: a fee is mandatorily explicit, so it
    // contributes a zero blinder and there is nothing left to absorb a
    // nonzero input blinder sum. The floor turns it away on cardinality
    // first, which is the same wall it always met.
    let mut sole_fee = fee_bearing_manifest(Vec::new());
    sole_fee.outputs.remove(0);
    assert_eq!(
        ConfidentialFixtureRegistry::new()
            .register(sole_fee)
            .expect_err("a lone fee output is refused"),
        RegistrationRefusal::OutputSetTooSmall { found: 1 },
    );

    // And a manifest of nothing but fee outputs states no solving output
    // at all, which is the refusal that names the real reason.
    let mut all_fees = fee_bearing_manifest(Vec::new());
    all_fees.outputs[0].role = FixtureOutputRole::Fee;
    all_fees.outputs[0].output_program = Vec::new();
    assert_eq!(
        ConfidentialFixtureRegistry::new()
            .register(all_fees)
            .expect_err("no output solves the balance"),
        RegistrationRefusal::BalancingRoleNotUnique { found: 0 },
    );
}

#[test]
fn a_zero_valued_fee_is_refused_like_any_other_output() {
    // The consensus fact this holds: a fee must be nonzero. A zero-value
    // explicit output is admitted at the target only where its script is
    // unspendable, and an empty script is not unspendable, so a zero-value
    // fee output is refused there outright. The registry never builds one,
    // and the clause that stops it is the positive-amount clause every
    // output already meets.
    let mut zero_fee = fee_bearing_manifest(Vec::new());
    zero_fee.outputs[1].semantic_amount = 0;
    assert_eq!(
        ConfidentialFixtureRegistry::new()
            .register(zero_fee)
            .expect_err("a zero-value fee is refused"),
        RegistrationRefusal::AmountNotPositive { output: 1 },
    );
}

#[test]
fn the_dual_parity_rule_reads_on_commitments_and_not_on_output_count() {
    // A two-OUTPUT fixture of one blinded output beside a fee is not the
    // dual-parity case, whatever its output count says: it has one
    // commitment, and the admitted pair in fixed order is a rule about
    // two. Had the rule kept counting outputs, this manifest would have
    // been required to produce a second prefix it has no second
    // commitment to carry, and the bounded search would have exhausted.
    ConfidentialFixtureRegistry::new()
        .register(fee_bearing_manifest(Vec::new()))
        .expect("a fee-bearing two-output manifest is not held to the pair");

    // And the genuine dual-parity case is still held to it, which the
    // predecessor's own fixed-order test asserts next door.
    let registry = frozen();
    let commitments = registry
        .resolve(&predecessor_handle(), &{
            *registry
                .registered_digest(&predecessor_handle())
                .expect("its own digest")
        })
        .expect("it resolves")
        .value_commitments()
        .expect("byte identity carries openings");
    assert_eq!(commitments.len(), 2, "two blinded outputs, two commitments");
}

// --- The explicit destination role, and the exit crossing it serves ----

/// One manifest of an EXIT CROSSING: two explicit receipt destinations
/// beside the blinded absorber a nonzero input blinder sum requires.
///
/// The absorber is an ordinary `Balancing` output at the LAST
/// destination position, which is the declaration the covenant checks
/// positionally. It is not a fourth output family and needs no second
/// balancing election.
fn exit_crossing_manifest(
    handle: &str,
    input_blinder_sum: [u8; 32],
) -> ConfidentialFixtureManifest {
    ConfidentialFixtureManifest {
        handle: ConfidentialFixtureHandle::new(handle.to_owned()),
        material_class: PublicDisposableTestMaterial::EXPECTED,
        derivation_profile: FixtureDerivationProfile::GuideCtfV1,
        profiles: profiles(ReproducibilityContract::ByteIdentity),
        retry_limit: MAX_PARITY_COUNTER,
        explicit_asset: ASSET,
        input_blinder_sum,
        outputs: vec![
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::ExplicitDestination,
                semantic_amount: 300_000_000,
                output_program: vec![0x51],
            },
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::ExplicitDestination,
                semantic_amount: 200_000_000,
                output_program: vec![0x52],
            },
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Balancing,
                semantic_amount: 100_000_000,
                output_program: vec![0x53],
            },
        ],
    }
}

#[test]
fn an_exit_crossing_solves_the_absorber_to_the_input_blinder_sum_itself() {
    // THE CLAIM THE EXIT CROSSING RESTS ON, recomputed rather than
    // argued: with no output deriving a blinder there is nothing to
    // subtract, so the solved absorber blinder is the input blinder sum
    // ITSELF. That makes the all-explicit-siblings case the EASIEST one
    // for the solve rather than a stretch of it -- the same arithmetic
    // the declared single-output form performs, reached through the
    // ordinary balancing role because the manifest has more than one
    // output.
    let mut registry = ConfidentialFixtureRegistry::new();
    registry
        .register(exit_crossing_manifest(
            "ctf-v1/exit-crossing",
            NON_CANCELING_SUM,
        ))
        .expect("an exit crossing registers");
    let frozen = registry.freeze();
    let handle = ConfidentialFixtureHandle::new("ctf-v1/exit-crossing".to_owned());
    let digest = *frozen
        .registered_digest(&handle)
        .expect("the registry holds its own digest");
    let resolved = frozen.resolve(&handle, &digest).expect("it resolves");

    let FixtureOpenings::Derived { openings, .. } = resolved.openings() else {
        panic!("byte identity derives openings");
    };
    assert_eq!(openings.len(), 3, "three outputs, three opening slots");

    // The two explicit destinations carry NO opening, exactly as a fee
    // carries none, and their blinder is the all-zero one every explicit
    // value is committed with.
    for index in [0_usize, 1] {
        assert!(
            openings[index].is_none(),
            "an explicit destination carries no opening, so slot {index} is absent",
        );
    }

    // The absorber's blinder is the input blinder sum, unchanged.
    let absorber = openings[2]
        .as_ref()
        .expect("the absorber carries the one opening");
    assert_eq!(
        absorber.value_blinder, NON_CANCELING_SUM,
        "with nothing derived to subtract, the solve returns the input blinder sum ITSELF",
    );
}

#[test]
fn an_exit_crossing_over_a_canceling_predecessor_is_refused_rather_than_built() {
    // The degeneracy, and it is the SAME one the register already
    // refuses: a predecessor whose consumed blinders cancel presents a
    // zero sum, the absorber's solved blinder is zero, and an absorber
    // hiding nothing is not an absorber. So an exit crossing must be run
    // over a NON-CANCELING predecessor, and that is arithmetic rather
    // than a preference.
    assert_eq!(
        ConfidentialFixtureRegistry::new()
            .register(exit_crossing_manifest("ctf-v1/exit-canceling", [0_u8; 32]))
            .expect_err("a zero input blinder sum leaves nothing to absorb"),
        RegistrationRefusal::Derivation {
            refusal: FixtureDerivationRefusal::DegenerateBalancingScalar,
        },
    );
}

#[test]
fn a_fully_unblinding_manifest_names_no_output_to_solve() {
    // THE CORNER THE CENSUS KEEPS AND THE WAVE DOES NOT ATTEMPT.
    // Dropping the absorber leaves a manifest of explicit destinations
    // only, and consensus admits that shape ONLY over a zero-sum input
    // set. The registry refuses it independently and for a reason of its
    // own -- no output solves the balance -- which is exactly the case
    // the register exists to keep apart from a consensus refusal.
    let mut fully_explicit = exit_crossing_manifest("ctf-v1/fully-unblinding", NON_CANCELING_SUM);
    fully_explicit.outputs[2].role = FixtureOutputRole::ExplicitDestination;
    assert_eq!(
        ConfidentialFixtureRegistry::new()
            .register(fully_explicit)
            .expect_err("no output solves the balance"),
        RegistrationRefusal::BalancingRoleNotUnique { found: 0 },
    );
}

#[test]
fn an_explicit_destination_must_carry_a_program_and_a_fee_must_not() {
    // The two non-opening roles are told apart by the ONE predicate that
    // differs, and each is held to its own side of it. This is what
    // stops an explicit receipt destination being registered as the
    // target's fee -- which would be registering a spendable output as
    // the thing a node treats as paid-away value.
    let mut programless = exit_crossing_manifest("ctf-v1/exit-programless", NON_CANCELING_SUM);
    programless.outputs[0].output_program = Vec::new();
    assert_eq!(
        ConfidentialFixtureRegistry::new()
            .register(programless)
            .expect_err("an explicit destination with no program is not a destination"),
        RegistrationRefusal::OutputProgramEmpty { output: 0 },
    );

    // And the mirror: the fee still may not carry one.
    assert_eq!(
        ConfidentialFixtureRegistry::new()
            .register(fee_bearing_manifest(vec![0x51]))
            .expect_err("a fee carrying a program is not a fee"),
        RegistrationRefusal::FeeProgramNotEmpty { output: 1 },
    );
}

#[test]
fn the_new_role_moves_no_recorded_digest_because_it_rides_a_new_code() {
    // The stability argument as arithmetic rather than as prose. Every
    // role's transcript code is distinct and the new one is the next
    // unused value, so no manifest registered before this role existed
    // can hash a byte differently -- there is no framing, field or flag
    // it shares with them that it changed.
    let codes = [
        FixtureOutputRole::Primary,
        FixtureOutputRole::Balancing,
        FixtureOutputRole::SoleBalancing,
        FixtureOutputRole::Fee,
        FixtureOutputRole::SponsorChange { asset: ASSET },
        FixtureOutputRole::ExplicitDestination,
        FixtureOutputRole::ExplicitSponsorChange { asset: ASSET },
        FixtureOutputRole::BalancingSponsorChange { asset: ASSET },
    ]
    .map(FixtureOutputRole::transcript_code);
    assert_eq!(
        codes,
        [1, 2, 3, 4, 5, 6, 7, 8],
        "codes are stable and each new one is next"
    );

    // And it takes the fee's side of every predicate but the program
    // one, which is the whole of what it is.
    let role = FixtureOutputRole::ExplicitDestination;
    assert!(!role.solves_the_balance());
    assert!(!role.carries_an_opening());
    assert!(!role.requires_an_empty_program());
    assert_eq!(role.own_asset(), None, "it takes the manifest's asset");
}

// --- The explicit sponsor change, and the wall it removes -------------

/// The reserve asset a sponsor's change is denominated in.
///
/// Public disposable test material under ADR-015; it names no chain. It
/// differs from `ASSET` because the whole point of a sponsor region is
/// that it moves a DIFFERENT asset.
const RESERVE_ASSET: [u8; 32] = [0x5c; 32];

/// A sponsored confidential manifest: one blinded destination, the
/// sponsor's change under `change`, and the fee.
fn sponsored_change_manifest(
    handle: &str,
    change: FixtureOutputRole,
) -> ConfidentialFixtureManifest {
    ConfidentialFixtureManifest {
        handle: ConfidentialFixtureHandle::new(handle.to_owned()),
        material_class: PublicDisposableTestMaterial::EXPECTED,
        derivation_profile: FixtureDerivationProfile::GuideCtfV1,
        profiles: profiles(ReproducibilityContract::ByteIdentity),
        retry_limit: MAX_PARITY_COUNTER,
        explicit_asset: ASSET,
        input_blinder_sum: NON_CANCELING_SUM,
        outputs: vec![
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Balancing,
                semantic_amount: 300_000_000,
                output_program: vec![0x51],
            },
            ConfidentialFixtureOutput {
                role: change,
                semantic_amount: 200_000_000,
                output_program: vec![0x52],
            },
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Fee,
                semantic_amount: 100_000_000,
                output_program: Vec::new(),
            },
        ],
    }
}

#[test]
fn an_explicit_sponsor_change_registers_and_carries_no_opening() {
    // The wall this member removes: for as long as the vocabulary's one
    // sponsor role was the committed change, a confidential manifest
    // whose sponsor takes explicit change could not be stated at all.
    // It now registers by NAMING the role, which is the whole of the
    // change -- nothing is inferred from a count or an amount.
    let mut registry = ConfidentialFixtureRegistry::new();
    registry
        .register(sponsored_change_manifest(
            "ctf-v1/explicit-sponsor-change",
            FixtureOutputRole::ExplicitSponsorChange {
                asset: RESERVE_ASSET,
            },
        ))
        .expect("a sponsored manifest taking explicit change registers");
    let frozen = registry.freeze();
    let handle = ConfidentialFixtureHandle::new("ctf-v1/explicit-sponsor-change".to_owned());
    let digest = *frozen
        .registered_digest(&handle)
        .expect("the registry holds its own digest");
    let resolved = frozen.resolve(&handle, &digest).expect("it resolves");

    let FixtureOpenings::Derived { openings, .. } = resolved.openings() else {
        panic!("byte identity derives openings");
    };
    assert_eq!(openings.len(), 3, "three outputs, three opening slots");
    assert!(
        openings[1].is_none(),
        "an explicit sponsor change carries NO opening, recorded as an absence",
    );

    // The parity rule read it as the explicit output it is: it
    // contributed a zero blinder, so the solved balancing blinder is
    // the input blinder sum itself, nothing having been derived to
    // subtract.
    let balancing = openings[0]
        .as_ref()
        .expect("the blinded destination carries the one opening");
    assert_eq!(
        balancing.value_blinder, NON_CANCELING_SUM,
        "the explicit change contributes zero to the solve",
    );

    // And the asset seam is the role's, not the manifest's.
    let change = FixtureOutputRole::ExplicitSponsorChange {
        asset: RESERVE_ASSET,
    };
    assert_eq!(change.own_asset(), Some(RESERVE_ASSET));
    assert!(!change.carries_an_opening());
    assert!(!change.solves_the_balance());
    assert!(!change.requires_an_empty_program());
}

#[test]
fn an_explicit_sponsor_change_is_held_to_the_clauses_of_an_explicit_output() {
    // The removal is a narrowing, not a relaxation: the new member is
    // read by the existing clauses exactly as its predicates answer,
    // and no clause was loosened to admit it.
    //
    // The empty-program clause reads it as a payment back to a real
    // owner, so a programless one draws the same refusal a programless
    // destination draws.
    let mut programless = sponsored_change_manifest(
        "ctf-v1/explicit-change-programless",
        FixtureOutputRole::ExplicitSponsorChange {
            asset: RESERVE_ASSET,
        },
    );
    programless.outputs[1].output_program = Vec::new();
    assert_eq!(
        ConfidentialFixtureRegistry::new()
            .register(programless)
            .expect_err("an explicit sponsor change with no program is refused"),
        RegistrationRefusal::OutputProgramEmpty { output: 1 },
    );

    // And the parity rule counts it as solving NOTHING: a manifest
    // whose outputs are the explicit change, an explicit destination
    // and the fee has no solving role and draws the refusal such a
    // manifest has always drawn, with the same count.
    let mut unsolved = sponsored_change_manifest(
        "ctf-v1/explicit-change-unsolved",
        FixtureOutputRole::ExplicitSponsorChange {
            asset: RESERVE_ASSET,
        },
    );
    unsolved.outputs[0].role = FixtureOutputRole::ExplicitDestination;
    assert_eq!(
        ConfidentialFixtureRegistry::new()
            .register(unsolved)
            .expect_err("no output solves the balance"),
        RegistrationRefusal::BalancingRoleNotUnique { found: 0 },
    );
}

// --- The balancing sponsor change, and the wall it removes ------------

/// A manifest whose ONLY blinded output is the sponsor's change: two
/// explicit destinations, the change under `change`, and the fee.
///
/// The form the solving-role wall held. Consensus admits it — the
/// change is a blinded output and the tally counts blinded outputs —
/// and for as long as no solving role could be stated outside the
/// destinations, no manifest of this form could register.
fn sole_blinded_change_manifest(
    handle: &str,
    change: FixtureOutputRole,
    input_blinder_sum: [u8; 32],
) -> ConfidentialFixtureManifest {
    ConfidentialFixtureManifest {
        handle: ConfidentialFixtureHandle::new(handle.to_owned()),
        material_class: PublicDisposableTestMaterial::EXPECTED,
        derivation_profile: FixtureDerivationProfile::GuideCtfV1,
        profiles: profiles(ReproducibilityContract::ByteIdentity),
        retry_limit: MAX_PARITY_COUNTER,
        explicit_asset: ASSET,
        input_blinder_sum,
        outputs: vec![
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::ExplicitDestination,
                semantic_amount: 300_000_000,
                output_program: vec![0x51],
            },
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::ExplicitDestination,
                semantic_amount: 200_000_000,
                output_program: vec![0x52],
            },
            ConfidentialFixtureOutput {
                role: change,
                semantic_amount: 150_000_000,
                output_program: vec![0x53],
            },
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Fee,
                semantic_amount: 50_000_000,
                output_program: Vec::new(),
            },
        ],
    }
}

#[test]
fn a_balancing_sponsor_change_solves_and_is_committed_against_its_own_asset() {
    // The wall this member removes: a form whose only blinded output is
    // the sponsor's change had no solving role to name. It now names
    // one, and the solve is the sole form's arithmetic reached through
    // a sponsor role: nothing derives, so the solved blinder is the
    // input blinder sum itself.
    let mut registry = ConfidentialFixtureRegistry::new();
    registry
        .register(sole_blinded_change_manifest(
            "ctf-v1/solving-sponsor-change",
            FixtureOutputRole::BalancingSponsorChange {
                asset: RESERVE_ASSET,
            },
            NON_CANCELING_SUM,
        ))
        .expect("a form whose only blinded output is the sponsor's change registers");
    let frozen = registry.freeze();
    let handle = ConfidentialFixtureHandle::new("ctf-v1/solving-sponsor-change".to_owned());
    let digest = *frozen
        .registered_digest(&handle)
        .expect("the registry holds its own digest");
    let resolved = frozen.resolve(&handle, &digest).expect("it resolves");

    let FixtureOpenings::Derived { openings, .. } = resolved.openings() else {
        panic!("byte identity derives openings");
    };
    assert_eq!(openings.len(), 4, "four outputs, four opening slots");
    for index in [0_usize, 1, 3] {
        assert!(
            openings[index].is_none(),
            "slot {index} is explicit and carries no opening",
        );
    }
    let change = openings[2]
        .as_ref()
        .expect("the solving change carries the one opening");
    assert_eq!(
        change.value_blinder, NON_CANCELING_SUM,
        "with nothing derived to subtract, the solve returns the input blinder sum ITSELF",
    );

    // The commitment is built against the role's OWN asset generator,
    // not the manifest's: a remainder committed against the protocol
    // asset would be a point the target reads as the wrong asset
    // entirely. Recomputed rather than asserted from the role.
    let recomputed = crate::commitment_oracle::commitment::commitment(
        &RESERVE_ASSET,
        150_000_000,
        &change.value_blinder,
    )
    .expect("the oracle commits");
    assert_eq!(
        change.value_commitment, recomputed,
        "the solving change is committed against the sponsor's reserve asset",
    );
}

#[test]
fn a_committed_change_that_does_not_declare_the_solving_role_draws_its_old_refusal() {
    // NARROWING, NOT RELAXATION, observed. The manifest the wall held
    // -- explicit destinations, a committed sponsor change stated with
    // the role that existed, a fee -- draws exactly the refusal it has
    // always drawn, with the same count. A form is freed only by
    // NAMING the solving member.
    assert_eq!(
        ConfidentialFixtureRegistry::new()
            .register(sole_blinded_change_manifest(
                "ctf-v1/undeclared-solving-change",
                FixtureOutputRole::SponsorChange {
                    asset: RESERVE_ASSET,
                },
                NON_CANCELING_SUM,
            ))
            .expect_err("an undeclared committed change still solves nothing"),
        RegistrationRefusal::BalancingRoleNotUnique { found: 0 },
    );

    // And the uniqueness clause counts the new member exactly as it
    // counts the old solving roles: declared beside a balancing
    // destination it draws the two-solvers refusal.
    let mut two_solvers = sole_blinded_change_manifest(
        "ctf-v1/two-solving-roles",
        FixtureOutputRole::BalancingSponsorChange {
            asset: RESERVE_ASSET,
        },
        NON_CANCELING_SUM,
    );
    two_solvers.outputs[0].role = FixtureOutputRole::Balancing;
    assert_eq!(
        ConfidentialFixtureRegistry::new()
            .register(two_solvers)
            .expect_err("two solving roles leave the sum underdetermined"),
        RegistrationRefusal::BalancingRoleNotUnique { found: 2 },
    );
}

#[test]
fn a_solving_sponsor_change_over_a_canceling_sum_is_refused_rather_than_built() {
    // The degeneracy warning travels WITH the solving role, which is
    // the T5-041 condition carried verbatim: a sole solved output over
    // a zero consumed sum hides nothing -- the solve returns the zero
    // unchanged and the commitment is exactly the value times the
    // asset generator. `DegenerateBalancingScalar` is what catches it,
    // left standing and load-bearing.
    assert_eq!(
        ConfidentialFixtureRegistry::new()
            .register(sole_blinded_change_manifest(
                "ctf-v1/solving-change-canceling",
                FixtureOutputRole::BalancingSponsorChange {
                    asset: RESERVE_ASSET,
                },
                [0_u8; 32],
            ))
            .expect_err("a zero consumed sum would hide nothing"),
        RegistrationRefusal::Derivation {
            refusal: FixtureDerivationRefusal::DegenerateBalancingScalar,
        },
    );
}
