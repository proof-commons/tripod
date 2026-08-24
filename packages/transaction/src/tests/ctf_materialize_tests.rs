//! The transaction-wide confidential materializer: one valid case and
//! the refusals an adversarial reading of its seven stages produces.
//!
//! # The arithmetic here is a stand-in, and says so
//!
//! This crate carries no bignum dependency and may not acquire one, so
//! the two injected collaborators below compute a stand-in "commitment"
//! from the same three inputs a real one takes. That makes the two
//! origins agree, which is what a valid case needs, and it makes these
//! tests measurements of the materializer's CONTROL FLOW rather than of
//! any cryptography.
//!
//! Nothing here is independence evidence and nothing here claims to be.
//! The real independent check is an adapter over the first-party bignum
//! commitment oracle, it lives in the one library that can see both
//! sides, and the tests that exercise it live there. What this file
//! establishes is that the stages run in order, that each refusal is
//! typed and reachable, and that the two origins are compared through a
//! function neither of them can be passed to twice.
//!
//! # Every value here is public test material
//!
//! Every scalar, program, asset, and commitment below is a meaningless
//! published byte string in the sense the workspace's test-material rule
//! fixes. None is a secret, none authorizes anything, and the word
//! "opening" here names a fixture rather than a key.

use std::collections::BTreeMap;

use target_elements::ReproducibilityContract;

use super::outpoint;
use crate::bytes::{AssetField, AssetId, COMMITMENT_BYTES, NonceField, Outpoint, ValueField};
use crate::live_materialize::{
    CommitmentOrigin, ConfidentialConstructionIntent, ConfidentialCustodyProfile,
    ConfidentialDestinationIntent, ConfidentialFixtureOutputView, ConfidentialFixtureView,
    ConfidentialInputIntent, ConfidentialMaterializationProfiles, ConfidentialMaterializerProfile,
    ConfidentialNonceProfile, ConfidentialOrderProfile, ConfidentialOutputRole,
    ConfidentialProofMaterializer, ConfidentialProofProfile, ConfidentialRetryProfile,
    DerivationRole, FamilyMember, FixtureOpeningReference, FrozenConfidentialFixtureView,
    IndependentCommitment, IndependentCommitmentCheck, MaterializationRefusal,
    MaterializedRangeproof, MaterializerCommitment, NonProtocolFundingRegion, NonProtocolMember,
    ParityOutcome, ProofFinalizedRegion, RangeproofRequest, SCALAR_BYTES,
    materialize_confidential_candidate,
};
use crate::live_private::{
    ConfidentialConstructionModel, PrivateConstructionNonClaim, SelectedConstructionModel,
};

/// The published disposable protocol asset.
const ASSET: [u8; 32] = [0x3b; 32];

/// A published asset that is not the protocol one.
const POLICY_ASSET: [u8; 32] = [0x9e; 32];

/// The predecessor fixture's public handle.
const PREDECESSOR: &str = "ctf-v1/wave-three-predecessor";

/// The successor fixture's public handle.
const SUCCESSOR: &str = "ctf-v1/wave-three-successor";

/// The predecessor fixture's registered drift digest.
const PREDECESSOR_DIGEST: [u8; 32] = [0x71; 32];

/// The successor fixture's registered drift digest.
const SUCCESSOR_DIGEST: [u8; 32] = [0x72; 32];

/// The consumed input's published value blinder.
const INPUT_BLINDER: [u8; SCALAR_BYTES] = [0x11; SCALAR_BYTES];

/// The first destination's published value blinder.
const PRIMARY_BLINDER: [u8; SCALAR_BYTES] = [0x22; SCALAR_BYTES];

/// The consumed input's semantic amount.
const CONSUMED: u64 = 1_000;

/// The first destination's semantic amount.
const PRIMARY_AMOUNT: u64 = 700;

/// The second destination's semantic amount.
const BALANCING_AMOUNT: u64 = 300;

/// The stand-in group operation the two stub origins share.
///
/// A real solve is the input blinder sum minus the other outputs' sum in
/// the group. This is the same shape over a group that is easier to write
/// down, and it is chosen so that the balancing blinder is a function of
/// the other two rather than a constant a test could have guessed.
fn stub_solve(
    input_blinder_sum: &[u8; SCALAR_BYTES],
    others: &[[u8; SCALAR_BYTES]],
) -> [u8; SCALAR_BYTES] {
    let mut solved = *input_blinder_sum;
    for other in others {
        for (slot, byte) in solved.iter_mut().zip(other) {
            *slot ^= *byte;
        }
    }
    solved
}

/// The balancing blinder the stub solve produces for the valid case.
fn balancing_blinder() -> [u8; SCALAR_BYTES] {
    stub_solve(&INPUT_BLINDER, &[PRIMARY_BLINDER])
}

/// The stand-in commitment both origins compute.
///
/// It depends on all three of the arguments a real commitment depends on,
/// which is the only property the tests below need: two different
/// openings must not produce the same field.
fn stub_commitment(
    asset: AssetId,
    amount: u64,
    blinder: &[u8; SCALAR_BYTES],
) -> [u8; COMMITMENT_BYTES] {
    let mut bytes = [0_u8; COMMITMENT_BYTES];
    bytes[0] = 0x08 | u8::from(amount % 2 == 1);
    for (index, slot) in bytes[1..].iter_mut().enumerate() {
        let amount_byte = u8::try_from(amount % 256).unwrap_or_default();
        let index_byte = u8::try_from(index % 256).unwrap_or_default();
        *slot = asset.internal()[index] ^ blinder[index] ^ amount_byte.wrapping_add(index_byte);
    }
    bytes
}

/// A proof materializer that answers the stand-in way, or deviates in
/// exactly one named respect.
///
/// One deviation at a time rather than a set of independent switches: a
/// test that turned two on at once would be measuring which refusal comes
/// first rather than whether each is reachable, and the ordering of the
/// stages is stated by the guide rather than discovered here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum StubMaterializer {
    /// Answer every question the stand-in way.
    #[default]
    Faithful,
    /// Refuse the commitment, as a degenerate scalar or an identity
    /// point would.
    RefuseCommitment,
    /// Refuse the nonce.
    RefuseNonce,
    /// Answer with a proof bound to a different commitment.
    MisbindProof,
    /// Answer with an empty proof.
    EmptyProof,
    /// Answer with a surjection proof the hybrid form forbids.
    SurjectionProof,
}

impl ConfidentialProofMaterializer for StubMaterializer {
    fn origin(&self) -> CommitmentOrigin {
        CommitmentOrigin::ConstructionMaterializer
    }

    fn value_commitment(
        &self,
        explicit_asset: AssetId,
        semantic_amount: u64,
        value_blinder: &[u8; SCALAR_BYTES],
    ) -> Option<MaterializerCommitment> {
        if *self == Self::RefuseCommitment {
            return None;
        }
        Some(MaterializerCommitment::from_materializer(stub_commitment(
            explicit_asset,
            semantic_amount,
            value_blinder,
        )))
    }

    fn nonce_commitment(&self, nonce_input: &[u8; SCALAR_BYTES]) -> Option<[u8; COMMITMENT_BYTES]> {
        if *self == Self::RefuseNonce {
            return None;
        }
        let mut bytes = [0_u8; COMMITMENT_BYTES];
        bytes[0] = 0x02;
        bytes[1..].copy_from_slice(nonce_input);
        Some(bytes)
    }

    fn range_proof(&self, request: &RangeproofRequest<'_>) -> Option<MaterializedRangeproof> {
        let proof = if *self == Self::EmptyProof {
            Vec::new()
        } else {
            let mut proof = vec![0xab_u8; 64];
            proof[0] = u8::try_from(request.output()).unwrap_or(u8::MAX);
            proof
        };
        let surjection = if *self == Self::SurjectionProof {
            vec![0xcd; 32]
        } else {
            Vec::new()
        };
        let bound = if *self == Self::MisbindProof {
            [0x5c; COMMITMENT_BYTES]
        } else {
            *request.value_commitment().bytes()
        };
        Some(MaterializedRangeproof::new(
            proof,
            surjection,
            bound,
            request.explicit_asset(),
            request.output_program().to_vec(),
        ))
    }
}

/// An independent check that answers the stand-in way, with a declared
/// origin a test may move.
#[derive(Clone, Copy, Debug)]
struct StubChecker {
    /// Which origin this check declares itself to be.
    origin: CommitmentOrigin,
    /// Disagree with the materializer about the commitment.
    disagree: bool,
    /// Refuse to solve the balancing blinder.
    refuse_solve: bool,
}

impl Default for StubChecker {
    fn default() -> Self {
        Self {
            origin: CommitmentOrigin::FirstPartyBignumOracle,
            disagree: false,
            refuse_solve: false,
        }
    }
}

impl IndependentCommitmentCheck for StubChecker {
    fn origin(&self) -> CommitmentOrigin {
        self.origin
    }

    fn recompute(
        &self,
        explicit_asset: AssetId,
        semantic_amount: u64,
        value_blinder: &[u8; SCALAR_BYTES],
    ) -> Option<IndependentCommitment> {
        let mut bytes = stub_commitment(explicit_asset, semantic_amount, value_blinder);
        if self.disagree {
            bytes[COMMITMENT_BYTES - 1] ^= 0xff;
        }
        Some(IndependentCommitment::from_independent_recomputation(bytes))
    }

    fn solve_balancing_blinder(
        &self,
        input_blinder_sum: &[u8; SCALAR_BYTES],
        other_blinders: &[[u8; SCALAR_BYTES]],
    ) -> Option<[u8; SCALAR_BYTES]> {
        if self.refuse_solve {
            return None;
        }
        Some(stub_solve(input_blinder_sum, other_blinders))
    }
}

/// The profiles every valid case selects.
const fn profiles() -> ConfidentialMaterializationProfiles {
    ConfidentialMaterializationProfiles {
        reproducibility_contract: ReproducibilityContract::ByteIdentity,
        custody_profile: ConfidentialCustodyProfile::CentralPublicFixtures,
        materializer_profile: ConfidentialMaterializerProfile::GuideCtfDeterministicV1,
        proof_profile: ConfidentialProofProfile::ExplicitAssetRangeproofV1,
        nonce_profile: ConfidentialNonceProfile::DeterministicDerivedV1,
        order_profile: ConfidentialOrderProfile::FixtureFixedOrder,
        retry_profile: ConfidentialRetryProfile::NoRetry,
    }
}

/// The protocol asset, as a typed identifier.
fn asset() -> AssetId {
    AssetId::from_internal(ASSET)
}

/// The outpoint the candidate consumes.
fn consumed() -> Outpoint {
    outpoint(0xc1, 0)
}

/// The predecessor fixture, whose one relevant output the candidate
/// consumes.
fn predecessor_fixture() -> ConfidentialFixtureView {
    ConfidentialFixtureView::new(
        PREDECESSOR_DIGEST,
        asset(),
        [0_u8; SCALAR_BYTES],
        ParityOutcome::Settled { counter: 0 },
        vec![ConfidentialFixtureOutputView::new(
            ConfidentialOutputRole::Primary,
            CONSUMED,
            vec![0x51],
            INPUT_BLINDER,
            [0x31; SCALAR_BYTES],
            [0x41; SCALAR_BYTES],
        )],
    )
}

/// The successor fixture, whose two outputs the candidate creates.
fn successor_fixture(
    balancing: [u8; SCALAR_BYTES],
    parity: ParityOutcome,
) -> ConfidentialFixtureView {
    ConfidentialFixtureView::new(
        SUCCESSOR_DIGEST,
        asset(),
        INPUT_BLINDER,
        parity,
        vec![
            ConfidentialFixtureOutputView::new(
                ConfidentialOutputRole::Primary,
                PRIMARY_AMOUNT,
                vec![0x51, 0x20, 0xaa],
                PRIMARY_BLINDER,
                [0x32; SCALAR_BYTES],
                [0x42; SCALAR_BYTES],
            ),
            ConfidentialFixtureOutputView::new(
                ConfidentialOutputRole::Balancing,
                BALANCING_AMOUNT,
                vec![0x51, 0x20, 0xbb],
                balancing,
                [0x33; SCALAR_BYTES],
                [0x43; SCALAR_BYTES],
            ),
        ],
    )
}

/// The frozen view the valid case resolves against.
fn view() -> FrozenConfidentialFixtureView {
    view_with(balancing_blinder(), ParityOutcome::Settled { counter: 0 })
}

/// The frozen view with the successor's balancing blinder and parity
/// outcome chosen.
fn view_with(
    balancing: [u8; SCALAR_BYTES],
    parity: ParityOutcome,
) -> FrozenConfidentialFixtureView {
    let mut entries = BTreeMap::new();
    entries.insert(PREDECESSOR.to_owned(), predecessor_fixture());
    entries.insert(SUCCESSOR.to_owned(), successor_fixture(balancing, parity));
    FrozenConfidentialFixtureView::new(entries)
}

/// The input the valid case consumes.
fn input() -> ConfidentialInputIntent {
    ConfidentialInputIntent::new(
        consumed(),
        AssetField::Explicit(asset()),
        ValueField::Commitment(stub_commitment(asset(), CONSUMED, &INPUT_BLINDER)),
        vec![0x51],
        0xffff_ffff,
        FixtureOpeningReference::new(PREDECESSOR.to_owned(), PREDECESSOR_DIGEST, 0),
        CONSUMED,
        [0_u8; SCALAR_BYTES],
    )
}

/// The two destinations the valid case creates.
fn destinations() -> Vec<ConfidentialDestinationIntent> {
    vec![
        ConfidentialDestinationIntent::new(
            PRIMARY_AMOUNT,
            asset(),
            vec![0x51, 0x20, 0xaa],
            FixtureOpeningReference::new(SUCCESSOR.to_owned(), SUCCESSOR_DIGEST, 0),
            ConfidentialOutputRole::Primary,
        ),
        ConfidentialDestinationIntent::new(
            BALANCING_AMOUNT,
            asset(),
            vec![0x51, 0x20, 0xbb],
            FixtureOpeningReference::new(SUCCESSOR.to_owned(), SUCCESSOR_DIGEST, 1),
            ConfidentialOutputRole::Balancing,
        ),
    ]
}

/// The valid intent.
fn intent() -> ConfidentialConstructionIntent {
    ConfidentialConstructionIntent::new(
        vec![input()],
        destinations(),
        NonProtocolFundingRegion::default(),
        profiles(),
        3,
        0,
    )
}

/// One materialization of `intent` against `view`.
fn materialize(
    intent: &ConfidentialConstructionIntent,
    view: &FrozenConfidentialFixtureView,
    crypto: &dyn ConfidentialProofMaterializer,
    checker: &dyn IndependentCommitmentCheck,
) -> Result<crate::live_materialize::MaterializedConfidentialCandidate, MaterializationRefusal> {
    materialize_confidential_candidate(intent, view, crypto, checker)
}

/// The valid case, materialized with the stand-in defaults.
fn valid() -> crate::live_materialize::MaterializedConfidentialCandidate {
    materialize(
        &intent(),
        &view(),
        &StubMaterializer::default(),
        &StubChecker::default(),
    )
    .expect("the valid intent materializes")
}

/// The refusal `intent` draws against the valid view and stubs.
fn refusal_of(intent: &ConfidentialConstructionIntent) -> MaterializationRefusal {
    materialize(
        intent,
        &view(),
        &StubMaterializer::default(),
        &StubChecker::default(),
    )
    .expect_err("the intent is refused")
}

// --- The valid case ----------------------------------------------------

/// One valid intent materializes a complete proof-bearing candidate.
///
/// The whole shape at once, because the shape is the deliverable: two
/// confidential protocol outputs carrying the explicit asset, committed
/// values, derived nonces, and a nonempty range proof each, with an empty
/// surjection field on both, frozen into one candidate whose bytes
/// round-trip.
#[test]
fn one_valid_intent_materializes_a_proof_bearing_candidate() {
    let candidate = valid();
    let protected = candidate.proof_finalized().protected();

    assert_eq!(protected.outputs().len(), 2, "two protocol outputs");
    assert_eq!(
        protected.output_witnesses().len(),
        2,
        "and one output-witness entry per output",
    );
    for (index, output) in protected.outputs().iter().enumerate() {
        assert_eq!(
            output.asset(),
            AssetField::Explicit(asset()),
            "every protocol member carries the explicit protocol asset",
        );
        assert!(
            matches!(output.value(), ValueField::Commitment(_)),
            "and a committed value",
        );
        assert!(
            matches!(output.nonce(), NonceField::Commitment(_)),
            "and a derived nonce, which the per-output role could not build",
        );
        let witness = &protected.output_witnesses()[index];
        assert!(!witness.range_proof().is_empty(), "and a range proof");
        assert!(
            witness.surjection_proof().is_empty(),
            "and an empty surjection field, which the hybrid form requires",
        );
    }
}

/// The frozen candidate's bytes round-trip exactly.
///
/// The wave's exit condition, checked on the materializer's own output
/// rather than on a hand-built transaction.
#[test]
fn a_proof_finalized_candidate_round_trips_exact_bytes() {
    let candidate = valid();
    let bytes = candidate.proof_finalized().protected().encode();
    let decoded = crate::bytes::TargetTransaction::decode(&bytes)
        .expect("the frozen candidate's bytes decode");

    assert_eq!(decoded.encode(), bytes, "and re-encode to themselves");
    assert_eq!(
        &decoded,
        candidate.proof_finalized().protected(),
        "and to the same typed value",
    );
}

/// The result carries a reference per input and no opening.
#[test]
fn the_result_carries_references_and_no_opening() {
    let candidate = valid();
    let census = candidate.opening_binding_census();

    assert_eq!(census.entries().len(), 1, "one reference per input");
    assert_eq!(census.entries()[0].handle(), PREDECESSOR);
    assert_eq!(census.entries()[0].outpoint(), consumed());

    assert_eq!(candidate.signer_inputs().len(), 1, "one signer input");
    let signer = &candidate.signer_inputs()[0];
    assert_eq!(
        signer.byte_binding(),
        candidate.proof_finalized().protected_bytes(),
        "bound to the exact protected bytes and not to a digest",
    );
    for opening in [INPUT_BLINDER, PRIMARY_BLINDER, balancing_blinder()] {
        assert!(
            !signer
                .byte_binding()
                .windows(SCALAR_BYTES)
                .any(|window| window == opening),
            "and carrying no opening",
        );
    }
}

// --- The protected preimage --------------------------------------------

/// Protected bytes change when one range-proof byte changes.
///
/// The sharpest test of the wave, and the one the whole protected-bytes
/// repair exists for. Two candidates differing in exactly one proof byte
/// must have different protected preimages; if they did not, an owner
/// would be binding to bytes that do not contain the proofs the target's
/// digest covers, and the resulting signature would be complete and
/// invalid.
///
/// The mutation is made by a materializer that answers one byte
/// differently, so what changed is a proof and nothing else — the
/// commitments, nonces, programs, amounts, version, and lock time are all
/// identical between the two runs.
#[test]
fn protected_bytes_change_when_one_range_proof_byte_changes() {
    /// A materializer whose proofs differ from the stub's in one byte.
    #[derive(Clone, Copy, Debug, Default)]
    struct OneByteDifferent(StubMaterializer);

    impl ConfidentialProofMaterializer for OneByteDifferent {
        fn origin(&self) -> CommitmentOrigin {
            self.0.origin()
        }

        fn value_commitment(
            &self,
            explicit_asset: AssetId,
            semantic_amount: u64,
            value_blinder: &[u8; SCALAR_BYTES],
        ) -> Option<MaterializerCommitment> {
            self.0
                .value_commitment(explicit_asset, semantic_amount, value_blinder)
        }

        fn nonce_commitment(
            &self,
            nonce_input: &[u8; SCALAR_BYTES],
        ) -> Option<[u8; COMMITMENT_BYTES]> {
            self.0.nonce_commitment(nonce_input)
        }

        fn range_proof(&self, request: &RangeproofRequest<'_>) -> Option<MaterializedRangeproof> {
            let answered = self.0.range_proof(request)?;
            let mut proof = answered.proof().to_vec();
            // One byte, at the tail, so that the proof's length and its
            // binding are both unchanged.
            let last = proof.len() - 1;
            proof[last] ^= 0x01;
            Some(MaterializedRangeproof::new(
                proof,
                answered.surjection_proof().to_vec(),
                *request.value_commitment().bytes(),
                request.explicit_asset(),
                request.output_program().to_vec(),
            ))
        }
    }

    let original = valid();
    let mutated = materialize_confidential_candidate(
        &intent(),
        &view(),
        &OneByteDifferent::default(),
        &StubChecker::default(),
    )
    .expect("the one-byte-different materializer also produces a candidate");

    let before = original.proof_finalized().protected_bytes();
    let after = mutated.proof_finalized().protected_bytes();

    assert_eq!(
        before.len(),
        after.len(),
        "the two preimages are the same length, so the difference is content",
    );
    assert_ne!(
        before, after,
        "and the preimage an owner binds to changed when a proof changed",
    );
    assert_eq!(
        original
            .proof_finalized()
            .protected()
            .encode_without_witness(),
        mutated
            .proof_finalized()
            .protected()
            .encode_without_witness(),
        "while the witnessless serialization did not, which is the omission the repair closes",
    );
    assert_eq!(
        original.proof_finalized().protected().outputs(),
        mutated.proof_finalized().protected().outputs(),
        "and no output field moved, so a proof is the only thing that differs",
    );
}

// --- Independence -------------------------------------------------------

/// An independent origin cannot self-attest.
///
/// Two ways at once, because the rule has two halves. A checker declaring
/// the materializer's own origin is refused, and so is a checker
/// declaring the reference implementation — the second because it binds
/// the same library the target vendors, so its agreement would be
/// conformance evidence wearing independence's name.
#[test]
fn an_independent_origin_cannot_self_attest() {
    for origin in [
        CommitmentOrigin::ConstructionMaterializer,
        CommitmentOrigin::ReferenceImplementation,
        CommitmentOrigin::TargetReadback,
    ] {
        let checker = StubChecker {
            origin,
            ..StubChecker::default()
        };
        assert_eq!(
            materialize(&intent(), &view(), &StubMaterializer::default(), &checker)
                .expect_err("a non-independent origin may not check a construction"),
            MaterializationRefusal::IndependentCommitmentOriginNotDistinct { output: 0 },
            "the refusal names the output the check was for",
        );
    }
}

/// Two origins that disagree about one commitment refuse.
#[test]
fn two_origins_that_disagree_refuse() {
    let checker = StubChecker {
        disagree: true,
        ..StubChecker::default()
    };

    assert_eq!(
        materialize(&intent(), &view(), &StubMaterializer::default(), &checker)
            .expect_err("a disagreement is not a candidate"),
        MaterializationRefusal::IndependentCommitmentMismatch { output: 0 },
    );
}

// --- Imbalance and blinders --------------------------------------------

/// Semantic imbalance and blinder imbalance are refused separately.
///
/// Two conditions and two refusals, which is the point: a build that
/// answered both with one word would leave a reader unable to tell an
/// arithmetic error in the amounts from an arithmetic error in the
/// blinders, and those are repaired in different places.
#[test]
fn semantic_imbalance_and_blinder_imbalance_are_refused_separately() {
    let mut destinations = destinations();
    destinations[0] = ConfidentialDestinationIntent::new(
        PRIMARY_AMOUNT + 1,
        asset(),
        vec![0x51, 0x20, 0xaa],
        FixtureOpeningReference::new(SUCCESSOR.to_owned(), SUCCESSOR_DIGEST, 0),
        ConfidentialOutputRole::Primary,
    );
    let imbalanced = ConfidentialConstructionIntent::new(
        vec![input()],
        destinations,
        NonProtocolFundingRegion::default(),
        profiles(),
        3,
        0,
    );
    // The order check runs first and would mask the imbalance, so the
    // fixture's own amount moves with the destination's.
    let mut entries = BTreeMap::new();
    entries.insert(PREDECESSOR.to_owned(), predecessor_fixture());
    let mut successor =
        successor_fixture(balancing_blinder(), ParityOutcome::Settled { counter: 0 });
    successor = ConfidentialFixtureView::new(
        SUCCESSOR_DIGEST,
        asset(),
        INPUT_BLINDER,
        successor.parity(),
        vec![
            ConfidentialFixtureOutputView::new(
                ConfidentialOutputRole::Primary,
                PRIMARY_AMOUNT + 1,
                vec![0x51, 0x20, 0xaa],
                PRIMARY_BLINDER,
                [0x32; SCALAR_BYTES],
                [0x42; SCALAR_BYTES],
            ),
            successor.outputs()[1].clone(),
        ],
    );
    entries.insert(SUCCESSOR.to_owned(), successor);

    assert_eq!(
        materialize_confidential_candidate(
            &imbalanced,
            &FrozenConfidentialFixtureView::new(entries),
            &StubMaterializer::default(),
            &StubChecker::default(),
        )
        .expect_err("the amounts do not close"),
        MaterializationRefusal::SemanticValueImbalance,
        "the semantic equation is named for itself",
    );

    let mut wrong_blinder = balancing_blinder();
    wrong_blinder[0] ^= 0xff;
    assert_eq!(
        materialize_confidential_candidate(
            &intent(),
            &view_with(wrong_blinder, ParityOutcome::Settled { counter: 0 }),
            &StubMaterializer::default(),
            &StubChecker::default(),
        )
        .expect_err("the blinders do not close"),
        MaterializationRefusal::ValueBlinderImbalance,
        "and the blinder equation is named for itself",
    );
}

/// An unsolvable balancing scalar is refused rather than nudged.
///
/// There is no arm here that adds one and tries again: a degenerate or
/// out-of-range solve refuses, and the refusal names the role the scalar
/// was for.
#[test]
fn an_invalid_balancing_scalar_is_refused() {
    let checker = StubChecker {
        refuse_solve: true,
        ..StubChecker::default()
    };

    assert_eq!(
        materialize(&intent(), &view(), &StubMaterializer::default(), &checker)
            .expect_err("an unsolvable balancing blinder is not a candidate"),
        MaterializationRefusal::InvalidScalar {
            role: DerivationRole::ValueBlinder,
        },
    );
}

/// A degenerate derived blinder is a typed refusal.
#[test]
fn a_degenerate_derived_blinder_is_refused() {
    let mut entries = BTreeMap::new();
    entries.insert(PREDECESSOR.to_owned(), predecessor_fixture());
    let successor = ConfidentialFixtureView::new(
        SUCCESSOR_DIGEST,
        asset(),
        INPUT_BLINDER,
        ParityOutcome::Settled { counter: 0 },
        vec![
            ConfidentialFixtureOutputView::new(
                ConfidentialOutputRole::Primary,
                PRIMARY_AMOUNT,
                vec![0x51, 0x20, 0xaa],
                [0_u8; SCALAR_BYTES],
                [0x32; SCALAR_BYTES],
                [0x42; SCALAR_BYTES],
            ),
            ConfidentialFixtureOutputView::new(
                ConfidentialOutputRole::Balancing,
                BALANCING_AMOUNT,
                vec![0x51, 0x20, 0xbb],
                balancing_blinder(),
                [0x33; SCALAR_BYTES],
                [0x43; SCALAR_BYTES],
            ),
        ],
    );
    entries.insert(SUCCESSOR.to_owned(), successor);

    assert_eq!(
        materialize_confidential_candidate(
            &intent(),
            &FrozenConfidentialFixtureView::new(entries),
            &StubMaterializer::default(),
            &StubChecker::default(),
        )
        .expect_err("zero is not a blinder"),
        MaterializationRefusal::InvalidScalar {
            role: DerivationRole::ValueBlinder,
        },
    );
}

/// A commitment that will not compute is a typed refusal.
#[test]
fn an_uncomputable_commitment_is_refused() {
    let crypto = StubMaterializer::RefuseCommitment;

    assert_eq!(
        materialize(&intent(), &view(), &crypto, &StubChecker::default())
            .expect_err("a commitment that will not compute is not a field"),
        MaterializationRefusal::InvalidCommitment { output: 0 },
    );
}

// --- Proofs -------------------------------------------------------------

/// An empty range proof is refused.
#[test]
fn an_empty_range_proof_is_refused() {
    let crypto = StubMaterializer::EmptyProof;

    assert_eq!(
        materialize(&intent(), &view(), &crypto, &StubChecker::default())
            .expect_err("an empty proof is refused by the target and here"),
        MaterializationRefusal::RangeproofEmpty { output: 0 },
    );
}

/// A proof bound to something else is refused.
#[test]
fn a_cross_bound_range_proof_is_refused() {
    let crypto = StubMaterializer::MisbindProof;

    assert_eq!(
        materialize(&intent(), &view(), &crypto, &StubChecker::default())
            .expect_err("a proof about another commitment proves nothing about this one"),
        MaterializationRefusal::ProofBindingMismatch { output: 0 },
    );
}

/// An unexpected surjection proof is refused.
#[test]
fn an_unexpected_surjection_proof_is_refused() {
    let crypto = StubMaterializer::SurjectionProof;

    assert_eq!(
        materialize(&intent(), &view(), &crypto, &StubChecker::default())
            .expect_err("the hybrid form requires the surjection field empty"),
        MaterializationRefusal::UnexpectedSurjectionProof { output: 0 },
    );
}

/// A proof failure adds no randomness and triggers no retry.
///
/// The refusal is the whole behaviour: there is one deterministic answer
/// and a failure to produce it ends the ceremony. The test measures that
/// by counting calls — a retry would show up as a second one.
#[test]
fn a_range_proof_failure_adds_no_randomness_and_triggers_no_retry() {
    use std::cell::Cell;

    /// A materializer that refuses every proof and counts the asking.
    struct Counting {
        inner: StubMaterializer,
        calls: Cell<usize>,
    }

    impl ConfidentialProofMaterializer for Counting {
        fn origin(&self) -> CommitmentOrigin {
            self.inner.origin()
        }

        fn value_commitment(
            &self,
            explicit_asset: AssetId,
            semantic_amount: u64,
            value_blinder: &[u8; SCALAR_BYTES],
        ) -> Option<MaterializerCommitment> {
            self.inner
                .value_commitment(explicit_asset, semantic_amount, value_blinder)
        }

        fn nonce_commitment(
            &self,
            nonce_input: &[u8; SCALAR_BYTES],
        ) -> Option<[u8; COMMITMENT_BYTES]> {
            self.inner.nonce_commitment(nonce_input)
        }

        fn range_proof(&self, _request: &RangeproofRequest<'_>) -> Option<MaterializedRangeproof> {
            self.calls.set(self.calls.get() + 1);
            None
        }
    }

    let crypto = Counting {
        inner: StubMaterializer::default(),
        calls: Cell::new(0),
    };

    assert_eq!(
        materialize_confidential_candidate(&intent(), &view(), &crypto, &StubChecker::default())
            .expect_err("a proof that will not generate is a refusal"),
        MaterializationRefusal::RangeproofMaterializationFailed { output: 0 },
    );
    assert_eq!(
        crypto.calls.get(),
        1,
        "asked once, refused once, and never asked again",
    );
}

/// Nonce material that will not derive is a typed refusal.
#[test]
fn nonce_material_that_will_not_derive_is_refused() {
    let crypto = StubMaterializer::RefuseNonce;

    assert_eq!(
        materialize(&intent(), &view(), &crypto, &StubChecker::default())
            .expect_err("a nonce that will not derive is not a field"),
        MaterializationRefusal::NonceMaterializationFailed { output: 0 },
    );
}

// --- Fixture binding and families ---------------------------------------

/// An unknown handle and a drifted digest both refuse before any
/// cryptographic work.
#[test]
fn an_unknown_handle_and_a_drifted_digest_both_refuse() {
    let unknown = ConfidentialConstructionIntent::new(
        vec![ConfidentialInputIntent::new(
            consumed(),
            AssetField::Explicit(asset()),
            ValueField::Commitment(stub_commitment(asset(), CONSUMED, &INPUT_BLINDER)),
            vec![0x51],
            0xffff_ffff,
            FixtureOpeningReference::new("ctf-v1/absent".to_owned(), PREDECESSOR_DIGEST, 0),
            CONSUMED,
            [0_u8; SCALAR_BYTES],
        )],
        destinations(),
        NonProtocolFundingRegion::default(),
        profiles(),
        3,
        0,
    );
    assert_eq!(
        refusal_of(&unknown),
        MaterializationRefusal::UnknownFixtureHandle {
            handle: "ctf-v1/absent".to_owned(),
        },
    );

    let drifted = ConfidentialConstructionIntent::new(
        vec![ConfidentialInputIntent::new(
            consumed(),
            AssetField::Explicit(asset()),
            ValueField::Commitment(stub_commitment(asset(), CONSUMED, &INPUT_BLINDER)),
            vec![0x51],
            0xffff_ffff,
            FixtureOpeningReference::new(PREDECESSOR.to_owned(), [0xee; 32], 0),
            CONSUMED,
            [0_u8; SCALAR_BYTES],
        )],
        destinations(),
        NonProtocolFundingRegion::default(),
        profiles(),
        3,
        0,
    );
    assert_eq!(
        refusal_of(&drifted),
        MaterializationRefusal::FixtureDigestMismatch {
            handle: PREDECESSOR.to_owned(),
        },
    );
}

/// An opening that does not recompute the observed commitment refuses.
#[test]
fn an_opening_that_does_not_recompute_the_observation_refuses() {
    let wrong = ConfidentialConstructionIntent::new(
        vec![ConfidentialInputIntent::new(
            consumed(),
            AssetField::Explicit(asset()),
            ValueField::Commitment([0x08; COMMITMENT_BYTES]),
            vec![0x51],
            0xffff_ffff,
            FixtureOpeningReference::new(PREDECESSOR.to_owned(), PREDECESSOR_DIGEST, 0),
            CONSUMED,
            [0_u8; SCALAR_BYTES],
        )],
        destinations(),
        NonProtocolFundingRegion::default(),
        profiles(),
        3,
        0,
    );

    assert_eq!(
        refusal_of(&wrong),
        MaterializationRefusal::PredecessorOpeningMismatch {
            outpoint: consumed(),
        },
    );
}

/// A reference past the fixture's own outputs is a missing opening.
#[test]
fn a_reference_past_the_fixtures_outputs_is_a_missing_opening() {
    let missing = ConfidentialConstructionIntent::new(
        vec![ConfidentialInputIntent::new(
            consumed(),
            AssetField::Explicit(asset()),
            ValueField::Commitment(stub_commitment(asset(), CONSUMED, &INPUT_BLINDER)),
            vec![0x51],
            0xffff_ffff,
            FixtureOpeningReference::new(PREDECESSOR.to_owned(), PREDECESSOR_DIGEST, 7),
            CONSUMED,
            [0_u8; SCALAR_BYTES],
        )],
        destinations(),
        NonProtocolFundingRegion::default(),
        profiles(),
        3,
        0,
    );

    assert_eq!(
        refusal_of(&missing),
        MaterializationRefusal::PredecessorOpeningMissing {
            outpoint: consumed(),
        },
    );
}

/// Two inputs on one outpoint refuse, and two bindings on one fixture
/// output refuse.
#[test]
fn duplicate_outpoints_and_ambiguous_bindings_both_refuse() {
    let duplicate = ConfidentialConstructionIntent::new(
        vec![input(), input()],
        destinations(),
        NonProtocolFundingRegion::default(),
        profiles(),
        3,
        0,
    );
    assert_eq!(
        refusal_of(&duplicate),
        MaterializationRefusal::DuplicateInputOutpoint {
            outpoint: consumed(),
        },
    );

    let mut second = input();
    second = ConfidentialInputIntent::new(
        outpoint(0xc2, 0),
        second.observed_asset(),
        second.observed_value(),
        second.observed_program().to_vec(),
        second.sequence(),
        FixtureOpeningReference::new(PREDECESSOR.to_owned(), PREDECESSOR_DIGEST, 0),
        CONSUMED,
        [0_u8; SCALAR_BYTES],
    );
    let ambiguous = ConfidentialConstructionIntent::new(
        vec![input(), second],
        destinations(),
        NonProtocolFundingRegion::default(),
        profiles(),
        3,
        0,
    );
    assert_eq!(
        refusal_of(&ambiguous),
        MaterializationRefusal::FixtureBindingAmbiguous {
            handle: PREDECESSOR.to_owned(),
        },
    );
}

/// The fixture's own order is the order, and a swap refuses.
#[test]
fn the_fixtures_own_order_is_the_order() {
    let mut swapped = destinations();
    swapped.swap(0, 1);
    let reordered = ConfidentialConstructionIntent::new(
        vec![input()],
        swapped,
        NonProtocolFundingRegion::default(),
        profiles(),
        3,
        0,
    );

    assert_eq!(
        refusal_of(&reordered),
        MaterializationRefusal::FixtureOutputOrderMismatch,
        "order is never fixed by a prefix, an amount, or a retry's finishing order",
    );
}

/// An exhausted parity search refuses.
#[test]
fn an_exhausted_parity_search_refuses() {
    assert_eq!(
        materialize_confidential_candidate(
            &intent(),
            &view_with(
                balancing_blinder(),
                ParityOutcome::Exhausted { attempts: 4096 }
            ),
            &StubMaterializer::default(),
            &StubChecker::default(),
        )
        .expect_err("an exhausted search produced no openings to build on"),
        MaterializationRefusal::BoundedParitySearchExhausted,
    );
}

/// An empty family is an incomplete classification.
#[test]
fn an_empty_family_is_an_incomplete_classification() {
    let no_inputs = ConfidentialConstructionIntent::new(
        Vec::new(),
        destinations(),
        NonProtocolFundingRegion::default(),
        profiles(),
        3,
        0,
    );
    assert_eq!(
        refusal_of(&no_inputs),
        MaterializationRefusal::IncompleteFamilyClassification,
    );

    let no_destinations = ConfidentialConstructionIntent::new(
        vec![input()],
        Vec::new(),
        NonProtocolFundingRegion::default(),
        profiles(),
        3,
        0,
    );
    assert_eq!(
        refusal_of(&no_destinations),
        MaterializationRefusal::IncompleteFamilyClassification,
    );
}

/// A committed protocol asset and a nonzero asset blinder both refuse.
#[test]
fn a_committed_protocol_asset_and_a_nonzero_asset_blinder_both_refuse() {
    let committed_asset = ConfidentialConstructionIntent::new(
        vec![ConfidentialInputIntent::new(
            consumed(),
            AssetField::Commitment([0x0a; COMMITMENT_BYTES]),
            ValueField::Commitment(stub_commitment(asset(), CONSUMED, &INPUT_BLINDER)),
            vec![0x51],
            0xffff_ffff,
            FixtureOpeningReference::new(PREDECESSOR.to_owned(), PREDECESSOR_DIGEST, 0),
            CONSUMED,
            [0_u8; SCALAR_BYTES],
        )],
        destinations(),
        NonProtocolFundingRegion::default(),
        profiles(),
        3,
        0,
    );
    assert_eq!(
        refusal_of(&committed_asset),
        MaterializationRefusal::ConfidentialProtocolAsset {
            member: FamilyMember::Input(0),
        },
    );

    let nonzero = ConfidentialConstructionIntent::new(
        vec![ConfidentialInputIntent::new(
            consumed(),
            AssetField::Explicit(asset()),
            ValueField::Commitment(stub_commitment(asset(), CONSUMED, &INPUT_BLINDER)),
            vec![0x51],
            0xffff_ffff,
            FixtureOpeningReference::new(PREDECESSOR.to_owned(), PREDECESSOR_DIGEST, 0),
            CONSUMED,
            [0x01; SCALAR_BYTES],
        )],
        destinations(),
        NonProtocolFundingRegion::default(),
        profiles(),
        3,
        0,
    );
    assert_eq!(
        refusal_of(&nonzero),
        MaterializationRefusal::NonzeroProtocolAssetBlinder {
            member: FamilyMember::Input(0),
        },
    );
}

/// A wrong protocol asset refuses and names its member.
#[test]
fn a_wrong_protocol_asset_refuses() {
    let wrong = ConfidentialConstructionIntent::new(
        vec![ConfidentialInputIntent::new(
            consumed(),
            AssetField::Explicit(AssetId::from_internal(POLICY_ASSET)),
            ValueField::Commitment(stub_commitment(asset(), CONSUMED, &INPUT_BLINDER)),
            vec![0x51],
            0xffff_ffff,
            FixtureOpeningReference::new(PREDECESSOR.to_owned(), PREDECESSOR_DIGEST, 0),
            CONSUMED,
            [0_u8; SCALAR_BYTES],
        )],
        destinations(),
        NonProtocolFundingRegion::default(),
        profiles(),
        3,
        0,
    );

    assert_eq!(
        refusal_of(&wrong),
        MaterializationRefusal::ProtocolAssetMismatch {
            member: FamilyMember::Input(0),
        },
    );
}

// --- The excluded region -------------------------------------------------

/// A non-protocol region really is excluded from both equations.
///
/// The valid case with a policy fee and a policy change output added: the
/// semantic equation still closes over the protocol region alone, and the
/// two extra outputs are serialized with empty output witnesses because
/// an explicit value admits no proof.
#[test]
fn the_non_protocol_region_is_excluded_from_both_equations() {
    let region = NonProtocolFundingRegion::new(vec![
        NonProtocolMember::new(AssetId::from_internal(POLICY_ASSET), 25, Vec::new()),
        NonProtocolMember::new(AssetId::from_internal(POLICY_ASSET), 900, vec![0x51, 0x99]),
    ]);
    let with_region = ConfidentialConstructionIntent::new(
        vec![input()],
        destinations(),
        region,
        profiles(),
        3,
        0,
    );

    let candidate = materialize_confidential_candidate(
        &with_region,
        &view(),
        &StubMaterializer::default(),
        &StubChecker::default(),
    )
    .expect("the protocol region closes without the excluded one");
    let protected = candidate.proof_finalized().protected();

    assert_eq!(protected.outputs().len(), 4, "two protocol, two excluded");
    for index in 2..4 {
        assert!(
            protected.output_witnesses()[index].is_empty(),
            "an explicit non-protocol output carries no proof",
        );
    }
}

/// A non-protocol member carrying the protocol asset refuses.
#[test]
fn a_non_protocol_member_carrying_the_protocol_asset_refuses() {
    let region =
        NonProtocolFundingRegion::new(vec![NonProtocolMember::new(asset(), 25, Vec::new())]);
    let overlapping = ConfidentialConstructionIntent::new(
        vec![input()],
        destinations(),
        region,
        profiles(),
        3,
        0,
    );

    assert_eq!(
        refusal_of(&overlapping),
        MaterializationRefusal::NonProtocolRegionAffectsProtocolBalance,
    );
}

/// A non-protocol member in a protocol member's place refuses.
#[test]
fn a_non_protocol_member_in_a_protocol_place_refuses() {
    let region = NonProtocolFundingRegion::new(vec![NonProtocolMember::new(
        AssetId::from_internal(POLICY_ASSET),
        25,
        vec![0x51, 0x20, 0xaa],
    )]);
    let overlapping = ConfidentialConstructionIntent::new(
        vec![input()],
        destinations(),
        region,
        profiles(),
        3,
        0,
    );

    assert_eq!(
        refusal_of(&overlapping),
        MaterializationRefusal::NonProtocolRegionOverlapsProtocol {
            member: FamilyMember::NonProtocolOutput(0),
        },
    );
}

// --- Profiles and the retired role --------------------------------------

/// A per-output substitute is refused before construction.
///
/// The retirement, expressed where it can be checked: selecting the
/// per-output role is a typed refusal rather than a silent substitution,
/// and it is refused before any cryptographic work happens.
#[test]
fn a_per_output_substitute_is_refused() {
    let substituted = ConfidentialConstructionIntent::new(
        vec![input()],
        destinations(),
        NonProtocolFundingRegion::default(),
        ConfidentialMaterializationProfiles {
            materializer_profile: ConfidentialMaterializerProfile::PerOutputValueCapability,
            ..profiles()
        },
        3,
        0,
    );

    assert_eq!(
        refusal_of(&substituted),
        MaterializationRefusal::PerOutputMaterializationRefused,
    );
}

/// An unsupported profile combination refuses rather than defaulting.
#[test]
fn an_unsupported_profile_combination_refuses() {
    let unsupported = ConfidentialConstructionIntent::new(
        vec![input()],
        destinations(),
        NonProtocolFundingRegion::default(),
        ConfidentialMaterializationProfiles {
            reproducibility_contract: ReproducibilityContract::RecordedRandomness,
            ..profiles()
        },
        3,
        0,
    );

    assert_eq!(
        refusal_of(&unsupported),
        MaterializationRefusal::UnsupportedProfileCombination,
        "an unsupported combination never defaults to a supported neighbour",
    );
}

/// The range-proof non-claim is retired for the confidential lane only.
///
/// Both halves in one test, because either alone would be misleading: the
/// proof-bearing record drops it, the ordinary record still carries it,
/// and the other three stay in force under both.
#[test]
fn the_range_proof_non_claim_is_retired_by_scope_and_not_by_wish() {
    let per_output = SelectedConstructionModel::record(ConfidentialConstructionModel::EXPECTED)
        .expect("the expected model records");
    let proof_bearing =
        SelectedConstructionModel::record_proof_bearing(ConfidentialConstructionModel::EXPECTED)
            .expect("the expected model records for the proof-bearing lane");

    assert!(
        per_output
            .non_claims()
            .contains(&PrivateConstructionNonClaim::NoRangeProofIsProducedOrChecked),
        "the per-output lane produces no proof and still says so",
    );
    assert!(
        !proof_bearing
            .non_claims()
            .contains(&PrivateConstructionNonClaim::NoRangeProofIsProducedOrChecked),
        "the transaction-wide lane produces and checks one, so it does not",
    );
    assert_eq!(
        proof_bearing.non_claims().len(),
        per_output.non_claims().len() - 1,
        "and exactly one non-claim moved",
    );
    for other in [
        PrivateConstructionNonClaim::NotAProductionMultiOwnerPrivacyProtocol,
        PrivateConstructionNonClaim::NoOpeningIsSecret,
        PrivateConstructionNonClaim::FieldFormSettledOnlyOnTheTarget,
    ] {
        assert!(
            proof_bearing.non_claims().contains(&other),
            "every other non-claim stands unchanged",
        );
    }
}

// --- Post-finalization ---------------------------------------------------

/// A proof-finalized candidate refuses mutation in each protected region.
///
/// The census is walked rather than sampled: all eight regions, through
/// every operation that names one, so a region added later without a
/// refusal fails this test rather than being discovered by a reviewer.
#[test]
fn a_proof_finalized_candidate_refuses_mutation_in_each_protected_region() {
    let candidate = valid();
    let frozen = candidate.proof_finalized();

    assert_eq!(
        frozen.regions().len(),
        ProofFinalizedRegion::ALL.len(),
        "every region the freeze rule names is protected",
    );

    for region in ProofFinalizedRegion::ALL.iter().copied() {
        for attempt in [
            frozen.insert(region),
            frozen.remove(region),
            frozen.replace(region),
            frozen.reorder(region),
        ] {
            assert_eq!(
                attempt.expect_err("a protected region does not move"),
                MaterializationRefusal::PostFinalizationMutation { region },
                "and the refusal names which region it was",
            );
        }
    }

    for (attempt, region) in [
        (
            frozen.regenerate_proofs(),
            ProofFinalizedRegion::OutputWitnesses,
        ),
        (
            frozen.repair_proofs(),
            ProofFinalizedRegion::OutputWitnesses,
        ),
        (frozen.reblind(), ProofFinalizedRegion::Outputs),
    ] {
        assert_eq!(
            attempt.expect_err("proof work is over at the freeze"),
            MaterializationRefusal::PostFinalizationMutation { region },
        );
    }
}

/// A signer input carrying an opening is refused.
///
/// Not a lint. A caller that passed an opening in as observed target data
/// would produce a signing input carrying it, and the way to keep that
/// from being reported is to refuse to construct the value at all.
#[test]
fn a_signer_input_that_would_carry_an_opening_is_refused() {
    let mut program = vec![0x51_u8];
    program.extend_from_slice(&PRIMARY_BLINDER);
    let leaking = ConfidentialConstructionIntent::new(
        vec![ConfidentialInputIntent::new(
            consumed(),
            AssetField::Explicit(asset()),
            ValueField::Commitment(stub_commitment(asset(), CONSUMED, &INPUT_BLINDER)),
            program,
            0xffff_ffff,
            FixtureOpeningReference::new(PREDECESSOR.to_owned(), PREDECESSOR_DIGEST, 0),
            CONSUMED,
            [0_u8; SCALAR_BYTES],
        )],
        destinations(),
        NonProtocolFundingRegion::default(),
        profiles(),
        3,
        0,
    );

    assert_eq!(
        refusal_of(&leaking),
        MaterializationRefusal::SignerInputWouldExposeOpening {
            outpoint: consumed(),
        },
    );
}
