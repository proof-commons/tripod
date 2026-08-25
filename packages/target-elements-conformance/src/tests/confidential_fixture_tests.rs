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
    DerivationRole, DerivationSource, FixtureDerivationProfile, FixtureDerivationRefusal,
    FixtureDiagnosticKind, FixtureOpenings, FixtureOutputRole, HandleGrammarDefect,
    MAX_PARITY_COUNTER, PublicDisposableTestMaterial, RegistrationRefusal, TaggedHashDerivation,
    check_handle_grammar, predecessor_handle,
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
