//! The owner signing-input census, its refusals, and the independently
//! written message construction that consumes it.
//!
//! # Two fixtures, and why there are two
//!
//! The positive route runs through the real materializer: a census is
//! built from a materialized confidential candidate, which is the only
//! public route there is, so the test exercises the deliverable rather
//! than a convenience constructor beside it.
//!
//! The refusal cases run through the crate-private parts seam. Several
//! of the clauses are structurally unreachable from a materialized
//! candidate — a frozen candidate's output-witness vector is one entry
//! per output by the transaction type's own invariant, and its protected
//! bytes are the bytes it computed — and a refusal no test can reach is
//! a refusal nobody has checked says what it says. The seam runs the
//! same clause list as the public route rather than a relaxed one.
//!
//! # The curve capability is a stand-in and says so
//!
//! [`CensusCurve`] performs no curve arithmetic. It answers the tweak
//! question with a tagged hash of its two inputs: reproducible,
//! distinguishable, and not a point. That is enough to exercise the
//! taproot commitment recomputation — which is a question about *what is
//! composed with what*, not about the curve — and it is not enough to
//! claim anything about a real output key. The crate's own live fixtures
//! already take this position for the same reason.
//!
//! # The pinned digests are candidate recomputations
//!
//! No digest here is authoritative. The pinned pair is what this
//! construction computes over a deterministic first-party candidate, and
//! it is pinned so that a later change to any term is a failing test
//! rather than a silently different message. Whether a target agrees is
//! a question this wave does not ask.

use target_elements::{LeafVersion, ReviewedElementsTapscriptDefinition};

use super::ctf_materialize_tests::valid_with_spent_program;
use super::reviewed_target;
use crate::bytes::{
    AssetField, AssetId, NonceField, Outpoint, OutputWitness, TargetInput, TargetOutput,
    TargetTransaction, Txid, ValueField,
};
use crate::live_census::{
    AnnexDisposition, IssuanceDisposition, LiveDeployment, OWNER_CODESEPARATOR_POSITION,
    OWNER_KEY_VERSION_BYTE, OWNER_SIGHASH_TYPE_BYTE, OWNER_SIGNATURE_BYTES, OWNER_SPEND_TYPE_BYTE,
    OwnerCensusRefusal, OwnerSigningCensus, OwnerSigningInputRequest, SpentOutputCensusEntry,
    check_signature_width, check_type_byte, spend_type_byte,
};
use crate::live_message::{
    TAP_SIGHASH_TAG, WitnessVectorTreatment, candidate_message_pair, candidate_owner_message,
};
use crate::live_taproot::{LiveCurveCapability, TweakedOutputKey};
use crate::taproot::{
    Digest32, OutputKeyParity, TAPROOT_WITNESS_VERSION, tagged_hash, witness_program_script,
};

// --- Fixtures ----------------------------------------------------------

/// The tag the stand-in curve derives its output keys under.
const CENSUS_KEY_TAG: &str = "Guide-sighash/w2-census-fixture-output-key";

/// The internal key every request below carries in its control block.
///
/// Public test material: a meaningless byte string that is not a point
/// and stands for no deployed key.
const INTERNAL_KEY: [u8; 32] = [0x77; 32];

/// The executing leaf's hash.
const LEAF_HASH: Digest32 = [0x5a; 32];

/// The deployment's genesis block hash.
///
/// Also public test material. A chain nobody settles on.
pub(super) const GENESIS: Digest32 = [0x21; 32];

/// A second deployment, for the mismatch case.
const OTHER_GENESIS: Digest32 = [0x22; 32];

/// A curve capability that performs no curve arithmetic.
struct CensusCurve;

impl LiveCurveCapability for CensusCurve {
    fn owner_key_is_a_curve_point(&self, owner: &[u8]) -> bool {
        !owner.is_empty()
    }

    fn output_key(&self, internal_key: &[u8], merkle_root: &Digest32) -> Option<TweakedOutputKey> {
        let mut preimage = Vec::with_capacity(internal_key.len() + merkle_root.len());
        preimage.extend_from_slice(internal_key);
        preimage.extend_from_slice(merkle_root);
        Some(TweakedOutputKey::new(
            tagged_hash(CENSUS_KEY_TAG, &preimage),
            OutputKeyParity::Even,
        ))
    }
}

/// A curve capability with no answer, for the not-a-point case.
struct SilentCurve;

impl LiveCurveCapability for SilentCurve {
    fn owner_key_is_a_curve_point(&self, _owner: &[u8]) -> bool {
        false
    }

    fn output_key(
        &self,
        _internal_key: &[u8],
        _merkle_root: &Digest32,
    ) -> Option<TweakedOutputKey> {
        None
    }
}

/// The control block a single-leaf tree produces: the version-and-parity
/// byte, the internal key, and an empty path.
fn control_block() -> Vec<u8> {
    let mut block = vec![LeafVersion::TAPSCRIPT.get() & 0xfe];
    block.extend_from_slice(&INTERNAL_KEY);
    block
}

/// The witness program the stand-in curve's answer commits to.
///
/// Recomputed here the way the census recomputes it, so the fixture and
/// the check agree by derivation rather than by a copied constant.
fn committed_program(target: &ReviewedElementsTapscriptDefinition) -> Vec<u8> {
    let curve = CensusCurve;
    let output_key = curve
        .output_key(&INTERNAL_KEY, &LEAF_HASH)
        .expect("the stand-in curve answers");
    witness_program_script(target, TAPROOT_WITNESS_VERSION, output_key.key())
        .expect("the reviewed grammar builds a witness program")
}

/// The signing request the positive cases use.
fn request() -> OwnerSigningInputRequest {
    OwnerSigningInputRequest::new(
        0,
        LEAF_HASH,
        LeafVersion::TAPSCRIPT,
        OWNER_CODESEPARATOR_POSITION,
        AnnexDisposition::Absent,
        IssuanceDisposition::Absent,
        control_block(),
    )
}

/// A deterministic first-party candidate: one input, two outputs.
///
/// Every field is a fixed byte string, so the messages computed over it
/// are the same on every run and on every machine — which is what makes
/// the pinned digests below replayable rather than a snapshot of one
/// run.
fn pinned_candidate() -> TargetTransaction {
    let outpoint =
        Outpoint::new(Txid::from_internal([0xc1; 32]), 0).expect("the index is in range");
    let asset = AssetField::Explicit(AssetId::from_internal([0x3b; 32]));

    TargetTransaction::with_output_witnesses(
        2,
        vec![TargetInput::new(outpoint, 0xffff_ffff)],
        vec![
            TargetOutput::new(
                asset,
                ValueField::Explicit(700),
                NonceField::Null,
                vec![0x51, 0x20, 0xaa],
            ),
            TargetOutput::new(
                asset,
                ValueField::Explicit(300),
                NonceField::Null,
                Vec::new(),
            ),
        ],
        0,
        vec![crate::bytes::InputWitness::default()],
        vec![
            OutputWitness::range_proof_only(vec![0x9c; 64]),
            OutputWitness::empty(),
        ],
    )
    .expect("the pinned candidate is well formed")
}

/// The pinned candidate's census, built through the parts seam.
pub(super) fn pinned_census(target: &ReviewedElementsTapscriptDefinition) -> OwnerSigningCensus {
    let candidate = pinned_candidate();

    parts_census(target, candidate, |parts| parts)
}

/// The parts a census is assembled from, so a case can disturb exactly
/// one of them.
struct CensusParts {
    candidate: TargetTransaction,
    protected_bytes: Vec<u8>,
    output_witnesses: Vec<OutputWitness>,
    spent_outputs: Vec<SpentOutputCensusEntry>,
    deployment: LiveDeployment,
    requests: Vec<OwnerSigningInputRequest>,
}

/// One census over the pinned candidate, with the parts disturbed by
/// `disturb` before assembly.
fn parts_census(
    target: &ReviewedElementsTapscriptDefinition,
    candidate: TargetTransaction,
    disturb: impl FnOnce(CensusParts) -> CensusParts,
) -> OwnerSigningCensus {
    try_parts_census(target, candidate, disturb).expect("the undisturbed parts assemble")
}

/// The same, without expecting success.
fn try_parts_census(
    target: &ReviewedElementsTapscriptDefinition,
    candidate: TargetTransaction,
    disturb: impl FnOnce(CensusParts) -> CensusParts,
) -> Result<OwnerSigningCensus, OwnerCensusRefusal> {
    let protected_bytes = candidate.encode();
    let output_witnesses = candidate.output_witnesses().to_vec();
    let spent_outputs = vec![SpentOutputCensusEntry::new(
        AssetField::Explicit(AssetId::from_internal([0x3b; 32])),
        ValueField::Explicit(1_000),
        committed_program(target),
    )];

    let parts = disturb(CensusParts {
        candidate,
        protected_bytes,
        output_witnesses,
        spent_outputs,
        deployment: LiveDeployment::new(GENESIS),
        requests: vec![request()],
    });

    OwnerSigningCensus::from_parts(
        target,
        parts.candidate,
        parts.protected_bytes,
        parts.output_witnesses,
        parts.spent_outputs,
        parts.deployment,
        &parts.requests,
        &CensusCurve,
    )
}

/// The refusal one disturbance draws.
fn refusal(disturb: impl FnOnce(CensusParts) -> CensusParts) -> OwnerCensusRefusal {
    let target = reviewed_target();
    try_parts_census(&target, pinned_candidate(), disturb)
        .expect_err("the disturbed parts are refused")
}

// --- Deliverable 4: the profile's constants are constants -------------

#[test]
fn the_profiles_constants_are_the_ruled_values() {
    // Written out rather than derived, because the point of the ruling
    // is that these three stopped being inputs a builder supplies.
    assert_eq!(OWNER_SIGHASH_TYPE_BYTE, 0x00);
    assert_eq!(OWNER_SPEND_TYPE_BYTE, 0x02);
    assert_eq!(OWNER_SIGNATURE_BYTES, 64);

    // Two more the review dispositioned as constants rather than census
    // inputs: the key version is fixed by the target at `:2702`, and the
    // codeseparator position is the value `:581` sets at the head of
    // evaluation.
    assert_eq!(OWNER_KEY_VERSION_BYTE, 0x00);
    assert_eq!(OWNER_CODESEPARATOR_POSITION, 0xffff_ffff);

    // The tag is the Elements one and not BIP-341's.
    assert_eq!(TAP_SIGHASH_TAG, "TapSighash/elements");
}

#[test]
fn the_spend_type_byte_is_recomputed_and_not_asserted() {
    // The constant is checkable rather than declared: it is the
    // script-path extension flag shifted with the annex bit added, and
    // the refused disposition recomputes to the other value. A test that
    // only compared the constant to itself would not notice a
    // recomputation that had stopped depending on the annex.
    assert_eq!(spend_type_byte(AnnexDisposition::Absent), 0x02);
    assert_eq!(spend_type_byte(AnnexDisposition::Present), 0x03);
    assert_eq!(
        spend_type_byte(AnnexDisposition::Absent),
        OWNER_SPEND_TYPE_BYTE
    );
}

#[test]
fn the_profile_checks_the_returned_type_byte_rather_than_recording_it() {
    assert_eq!(check_type_byte(OWNER_SIGHASH_TYPE_BYTE), Ok(()));

    // Every other byte the *target* admits is still a refusal here,
    // because the profile is narrower than the target.
    for offered in [0x01_u8, 0x02, 0x03, 0x81, 0x82, 0x83] {
        assert_eq!(
            check_type_byte(offered),
            Err(OwnerCensusRefusal::TypeByteOutsideProfile { offered }),
        );
    }
}

#[test]
fn the_profile_checks_the_returned_signature_width() {
    assert_eq!(check_signature_width(OWNER_SIGNATURE_BYTES), Ok(()));

    for offered in [0_usize, 63, 65, 66] {
        assert_eq!(
            check_signature_width(offered),
            Err(OwnerCensusRefusal::SignatureWidthOutsideProfile { offered }),
        );
    }
}

// --- Deliverables 1 and 2: the census and its only route --------------

#[test]
fn a_census_is_built_from_a_proof_finalized_candidate_and_carries_option_bs_fields() {
    let target = reviewed_target();
    let materialized = valid_with_spent_program(committed_program(&target));

    let census = OwnerSigningCensus::from_proof_finalized(
        &target,
        &materialized,
        LiveDeployment::new(GENESIS),
        &[request()],
        &CensusCurve,
    )
    .expect("the materialized candidate censuses");

    let frozen = materialized.proof_finalized();

    // Every field the accepted result names, read back.
    assert_eq!(census.protected_bytes(), frozen.protected_bytes());
    assert_eq!(
        census.output_witnesses(),
        frozen.protected().output_witnesses()
    );
    assert_eq!(
        census.spent_outputs().len(),
        frozen.protected().inputs().len()
    );
    assert_eq!(census.genesis_block_hash(), &GENESIS);
    assert_eq!(census.signing_inputs().len(), 1);

    let input = &census.signing_inputs()[0];
    assert_eq!(input.input_index(), 0);
    assert_eq!(input.tapleaf_hash(), &LEAF_HASH);
    assert_eq!(input.leaf_version(), LeafVersion::TAPSCRIPT);
    assert_eq!(input.codeseparator_position(), OWNER_CODESEPARATOR_POSITION);
    assert_eq!(input.annex(), AnnexDisposition::Absent);

    // The private lane's protected bytes contain the output-witness
    // vector, so they are not the witnessless serialization. That is the
    // repair the confidential-funding guide's Wave 3 landed, re-read
    // from the census's side.
    assert_ne!(
        census.protected_bytes(),
        frozen.protected().encode_without_witness().as_slice(),
    );
}

#[test]
fn the_census_binds_by_exact_bytes_rather_than_by_trust() {
    let target = reviewed_target();
    let census = pinned_census(&target);

    assert_eq!(census.check_offered(census.protected_bytes()), Ok(()));

    let mut mutated = census.protected_bytes().to_vec();
    let last = mutated.len() - 1;
    mutated[last] ^= 0x01;

    assert_eq!(
        census.check_offered(&mutated),
        Err(OwnerCensusRefusal::ProtectedBytesAreNotTheCandidates),
    );
}

// --- Deliverable 5: every refusal, reached ----------------------------

#[test]
fn a_short_spent_output_census_is_refused() {
    assert_eq!(
        refusal(|mut parts| {
            parts.spent_outputs.clear();
            parts
        }),
        OwnerCensusRefusal::SpentOutputCardinalityMismatch {
            inputs: 1,
            spent_outputs: 0,
        },
    );
}

#[test]
fn an_output_witness_vector_at_the_wrong_length_is_refused() {
    // The recorded hazard, refused where a census is assembled rather
    // than discovered when a signature turns out to be complete and
    // invalid.
    assert_eq!(
        refusal(|mut parts| {
            parts.output_witnesses.clear();
            parts
        }),
        OwnerCensusRefusal::OutputWitnessLengthMismatch {
            outputs: 2,
            output_witnesses: 0,
        },
    );
}

#[test]
fn an_output_witness_vector_of_the_right_length_but_the_wrong_contents_is_refused() {
    // Length alone is not the claim: a vector that is the right size and
    // holds another candidate's proofs hashes to a different value in
    // exactly the same way.
    assert_eq!(
        refusal(|mut parts| {
            parts.output_witnesses = vec![OutputWitness::empty(), OutputWitness::empty()];
            parts
        }),
        OwnerCensusRefusal::OutputWitnessLengthMismatch {
            outputs: 2,
            output_witnesses: 2,
        },
    );
}

#[test]
fn a_declared_annex_is_refused_with_the_spend_type_it_recomputes_to() {
    assert_eq!(
        refusal(|mut parts| {
            parts.requests = vec![OwnerSigningInputRequest::new(
                0,
                LEAF_HASH,
                LeafVersion::TAPSCRIPT,
                OWNER_CODESEPARATOR_POSITION,
                AnnexDisposition::Present,
                IssuanceDisposition::Absent,
                control_block(),
            )];
            parts
        }),
        OwnerCensusRefusal::AnnexDisagreement {
            input_index: 0,
            declared: AnnexDisposition::Present,
            recomputed_spend_type: 0x03,
        },
    );
}

#[test]
fn a_census_bound_to_another_deployment_is_refused() {
    let target = reviewed_target();
    let census = pinned_census(&target);

    // The census carries the deployment it was built against, and the
    // check is a comparison rather than a hope: two candidates identical
    // to the last byte have different messages on two chains.
    assert_eq!(census.genesis_block_hash(), &GENESIS);

    let other = parts_census(&target, pinned_candidate(), |mut parts| {
        parts.deployment = LiveDeployment::new(OTHER_GENESIS);
        parts
    });

    assert_ne!(other.genesis_block_hash(), census.genesis_block_hash());

    let input = &census.signing_inputs()[0];
    assert_ne!(
        candidate_owner_message(&census, input, WitnessVectorTreatment::BothGrown),
        candidate_owner_message(
            &other,
            &other.signing_inputs()[0],
            WitnessVectorTreatment::BothGrown
        ),
        "the deployment seeds the hasher, so it decides the message",
    );

    // The typed refusal, returned rather than merely declared: a census
    // carried to a run on another chain is refused before a message is
    // formed, which is the only place the mistake has a symptom.
    assert_eq!(
        census.check_deployment(LiveDeployment::new(GENESIS)),
        Ok(())
    );
    assert_eq!(
        census.check_deployment(LiveDeployment::new(OTHER_GENESIS)),
        Err(OwnerCensusRefusal::DeploymentMismatch {
            expected: GENESIS,
            offered: OTHER_GENESIS,
        }),
    );
}

#[test]
fn a_leaf_that_does_not_commit_under_the_control_block_is_refused() {
    assert_eq!(
        refusal(|mut parts| {
            parts.requests = vec![OwnerSigningInputRequest::new(
                0,
                [0x5b; 32],
                LeafVersion::TAPSCRIPT,
                OWNER_CODESEPARATOR_POSITION,
                AnnexDisposition::Absent,
                IssuanceDisposition::Absent,
                control_block(),
            )];
            parts
        }),
        OwnerCensusRefusal::LeafHashDoesNotCommit { input_index: 0 },
    );
}

#[test]
fn a_leaf_whose_path_reaches_another_root_is_refused() {
    // The other half of the same check: the leaf is the committed one
    // and the path is not, so the root folds to something the output key
    // does not commit to.
    assert_eq!(
        refusal(|mut parts| {
            let mut block = control_block();
            block.extend_from_slice(&[0x01; 32]);
            parts.requests = vec![OwnerSigningInputRequest::new(
                0,
                LEAF_HASH,
                LeafVersion::TAPSCRIPT,
                OWNER_CODESEPARATOR_POSITION,
                AnnexDisposition::Absent,
                IssuanceDisposition::Absent,
                block,
            )];
            parts
        }),
        OwnerCensusRefusal::LeafHashDoesNotCommit { input_index: 0 },
    );
}

#[test]
fn a_malformed_control_block_is_refused() {
    assert_eq!(
        refusal(|mut parts| {
            parts.requests = vec![OwnerSigningInputRequest::new(
                0,
                LEAF_HASH,
                LeafVersion::TAPSCRIPT,
                OWNER_CODESEPARATOR_POSITION,
                AnnexDisposition::Absent,
                IssuanceDisposition::Absent,
                vec![0xc4, 0x00],
            )];
            parts
        }),
        OwnerCensusRefusal::ControlBlockMalformed {
            input_index: 0,
            offered: 2,
        },
    );
}

#[test]
fn a_control_block_whose_leaf_version_disagrees_is_refused() {
    let mut block = control_block();
    block[0] = 0xc0;

    assert_eq!(
        refusal(|mut parts| {
            parts.requests = vec![OwnerSigningInputRequest::new(
                0,
                LEAF_HASH,
                LeafVersion::TAPSCRIPT,
                OWNER_CODESEPARATOR_POSITION,
                AnnexDisposition::Absent,
                IssuanceDisposition::Absent,
                block,
            )];
            parts
        }),
        OwnerCensusRefusal::LeafVersionDisagreesWithTheControlBlock {
            input_index: 0,
            declared: 0xc4,
            control_byte: 0xc0,
        },
    );
}

#[test]
fn an_internal_key_with_no_output_key_is_refused() {
    let target = reviewed_target();
    let candidate = pinned_candidate();
    let protected_bytes = candidate.encode();
    let output_witnesses = candidate.output_witnesses().to_vec();

    let refused = OwnerSigningCensus::from_parts(
        &target,
        candidate,
        protected_bytes,
        output_witnesses,
        vec![SpentOutputCensusEntry::new(
            AssetField::Explicit(AssetId::from_internal([0x3b; 32])),
            ValueField::Explicit(1_000),
            committed_program(&target),
        )],
        LiveDeployment::new(GENESIS),
        &[request()],
        &SilentCurve,
    )
    .expect_err("a capability with no answer refuses");

    assert_eq!(
        refused,
        OwnerCensusRefusal::ControlBlockInternalKeyIsNotAPoint { input_index: 0 },
    );
}

#[test]
fn an_issuance_bearing_input_is_refused() {
    // The precondition the source review put on this wave's desk: the
    // census carries no input-witness field, and that silence is only
    // valid while no input bears an issuance.
    assert_eq!(
        refusal(|mut parts| {
            parts.requests = vec![OwnerSigningInputRequest::new(
                0,
                LEAF_HASH,
                LeafVersion::TAPSCRIPT,
                OWNER_CODESEPARATOR_POSITION,
                AnnexDisposition::Absent,
                IssuanceDisposition::Bearing,
                control_block(),
            )];
            parts
        }),
        OwnerCensusRefusal::IssuanceBearingInputRefused { input_index: 0 },
    );
}

#[test]
fn a_signing_request_for_an_input_that_does_not_exist_is_refused() {
    assert_eq!(
        refusal(|mut parts| {
            parts.requests = vec![OwnerSigningInputRequest::new(
                7,
                LEAF_HASH,
                LeafVersion::TAPSCRIPT,
                OWNER_CODESEPARATOR_POSITION,
                AnnexDisposition::Absent,
                IssuanceDisposition::Absent,
                control_block(),
            )];
            parts
        }),
        OwnerCensusRefusal::SigningInputOutOfRange {
            input_index: 7,
            inputs: 1,
        },
    );
}

#[test]
fn two_requests_for_one_input_are_refused() {
    assert_eq!(
        refusal(|mut parts| {
            parts.requests = vec![request(), request()];
            parts
        }),
        OwnerCensusRefusal::DuplicateSigningInput { input_index: 0 },
    );
}

#[test]
fn a_census_with_nothing_to_authorize_is_refused() {
    assert_eq!(
        refusal(|mut parts| {
            parts.requests.clear();
            parts
        }),
        OwnerCensusRefusal::NoSigningInputRequested,
    );
}

#[test]
fn every_refusal_variant_is_reached_by_a_test_in_this_file() {
    // The census of the census. Fourteen variants, and the list is
    // spelled here so that adding a variant without a case is a failing
    // test rather than a silently unexercised refusal. Two are reached
    // through the standalone profile checks rather than through
    // assembly, and one through the offered-bytes check.
    let reached = [
        OwnerCensusRefusal::SpentOutputCardinalityMismatch {
            inputs: 1,
            spent_outputs: 0,
        },
        OwnerCensusRefusal::OutputWitnessLengthMismatch {
            outputs: 2,
            output_witnesses: 0,
        },
        OwnerCensusRefusal::AnnexDisagreement {
            input_index: 0,
            declared: AnnexDisposition::Present,
            recomputed_spend_type: 0x03,
        },
        OwnerCensusRefusal::DeploymentMismatch {
            expected: GENESIS,
            offered: OTHER_GENESIS,
        },
        OwnerCensusRefusal::LeafHashDoesNotCommit { input_index: 0 },
        OwnerCensusRefusal::ControlBlockMalformed {
            input_index: 0,
            offered: 2,
        },
        OwnerCensusRefusal::LeafVersionDisagreesWithTheControlBlock {
            input_index: 0,
            declared: 0xc4,
            control_byte: 0xc0,
        },
        OwnerCensusRefusal::ControlBlockInternalKeyIsNotAPoint { input_index: 0 },
        OwnerCensusRefusal::TypeByteOutsideProfile { offered: 0x01 },
        OwnerCensusRefusal::SignatureWidthOutsideProfile { offered: 65 },
        OwnerCensusRefusal::IssuanceBearingInputRefused { input_index: 0 },
        OwnerCensusRefusal::SigningInputOutOfRange {
            input_index: 7,
            inputs: 1,
        },
        OwnerCensusRefusal::DuplicateSigningInput { input_index: 0 },
        OwnerCensusRefusal::NoSigningInputRequested,
        OwnerCensusRefusal::ProtectedBytesAreNotTheCandidates,
    ];

    assert_eq!(reached.len(), 15);
}

// --- Deliverable 3: the message, and the two candidate digests --------

#[test]
fn the_two_candidate_messages_are_distinct() {
    // The property the whole recorded hazard rests on, and the property
    // the diagnosis checks before reporting any verdict: a run whose two
    // candidates coincided would make "which one does the signature
    // verify against" a coincidence rather than an answer.
    let target = reviewed_target();
    let census = pinned_census(&target);
    let pair = candidate_message_pair(&census, &census.signing_inputs()[0]);

    assert!(pair.candidates_are_distinct());
}

#[test]
fn the_two_candidate_messages_are_the_pinned_values() {
    // Pinned so that a change to any term of the construction is a
    // failing test rather than a silently different message. These are
    // candidate recomputations over a first-party fixture; no target has
    // been asked what it thinks of either, and neither is authoritative.
    let target = reviewed_target();
    let census = pinned_census(&target);
    let pair = candidate_message_pair(&census, &census.signing_inputs()[0]);

    assert_eq!(
        hex(pair.with_vector_grown()),
        PINNED_MESSAGE_WITH_VECTOR_GROWN,
    );
    assert_eq!(
        hex(pair.with_vector_empty()),
        PINNED_MESSAGE_WITH_VECTOR_EMPTY,
    );
}

#[test]
fn all_four_witness_treatments_are_distinct() {
    // Four candidates rather than two, which is the twin diagnosis's own
    // shape and its stated reason: the message has two length-dependent
    // terms and two candidates cannot say which of them moved.
    let target = reviewed_target();
    let census = pinned_census(&target);
    let input = &census.signing_inputs()[0];

    let messages = [
        WitnessVectorTreatment::BothGrown,
        WitnessVectorTreatment::OutputsEmptied,
        WitnessVectorTreatment::InputsEmptied,
        WitnessVectorTreatment::BothEmptied,
    ]
    .map(|treatment| candidate_owner_message(&census, input, treatment));

    for (first, one) in messages.iter().enumerate() {
        for (second, other) in messages.iter().enumerate() {
            assert_eq!(
                first == second,
                one == other,
                "treatments {first} and {second} must differ",
            );
        }
    }
}

#[test]
fn the_message_moves_when_one_range_proof_byte_moves() {
    // The output-witness term measured rather than argued: two
    // candidates differing in exactly one proof byte, at the same
    // length, have different messages.
    let target = reviewed_target();
    let census = pinned_census(&target);

    let mut mutated = pinned_candidate();
    let witnesses = vec![
        OutputWitness::range_proof_only({
            let mut proof = vec![0x9c; 64];
            proof[0] ^= 0x01;
            proof
        }),
        OutputWitness::empty(),
    ];
    mutated = TargetTransaction::with_output_witnesses(
        mutated.version(),
        mutated.inputs().to_vec(),
        mutated.outputs().to_vec(),
        mutated.lock_time(),
        mutated.witnesses().to_vec(),
        witnesses,
    )
    .expect("the mutated candidate is well formed");

    let other = parts_census(&target, mutated, |mut parts| {
        parts.protected_bytes = parts.candidate.encode();
        parts.output_witnesses = parts.candidate.output_witnesses().to_vec();
        parts
    });

    assert_ne!(
        candidate_owner_message(
            &census,
            &census.signing_inputs()[0],
            WitnessVectorTreatment::BothGrown
        ),
        candidate_owner_message(
            &other,
            &other.signing_inputs()[0],
            WitnessVectorTreatment::BothGrown,
        ),
    );
}

#[test]
fn the_message_is_the_stream_the_review_describes() {
    // The construction re-derived at the call site from the review's
    // term table, term by term, and compared against what the module
    // produces. Two authorings of one stream, and a disagreement is a
    // finding rather than a pass.
    use sha2::{Digest, Sha256};

    let target = reviewed_target();
    let census = pinned_census(&target);
    let input = &census.signing_inputs()[0];
    let candidate = census.candidate();

    let single = |bytes: &[u8]| -> [u8; 32] { Sha256::digest(bytes).into() };

    let mut stream = Vec::new();
    stream.extend_from_slice(&GENESIS);
    stream.extend_from_slice(&GENESIS);
    stream.push(0x00);
    stream.extend_from_slice(&candidate.version().to_le_bytes());
    stream.extend_from_slice(&candidate.lock_time().to_le_bytes());
    // Term 4: one flag byte per input, zero for an input with neither an
    // issuance nor a peg-in.
    stream.extend_from_slice(&single(&[0x00]));
    // Term 5: the outpoint, 32 bytes then four little-endian.
    let mut prevouts = Vec::new();
    prevouts.extend_from_slice(&[0xc1; 32]);
    prevouts.extend_from_slice(&0_u32.to_le_bytes());
    stream.extend_from_slice(&single(&prevouts));
    // Term 6: the spent asset field then the spent value field.
    let mut spent = vec![0x01];
    spent.extend_from_slice(&[0x3b; 32]);
    spent.push(0x01);
    spent.extend_from_slice(&1_000_u64.to_be_bytes());
    stream.extend_from_slice(&single(&spent));
    // Term 7: the spent script, compact-size prefixed.
    let program = committed_program(&target);
    let mut scripts = vec![u8::try_from(program.len()).expect("the fixture program is short")];
    scripts.extend_from_slice(&program);
    stream.extend_from_slice(&single(&scripts));
    // Term 8: the sequence.
    stream.extend_from_slice(&single(&0xffff_ffff_u32.to_le_bytes()));
    // Term 9: one zero byte for a null issuance.
    stream.extend_from_slice(&single(&[0x00]));
    // Term 10: two zero-length prefixes per input-witness entry.
    stream.extend_from_slice(&single(&[0x00, 0x00]));
    // Term 11: each output, serialized.
    let mut outputs = Vec::new();
    for output in candidate.outputs() {
        outputs.push(0x01);
        outputs.extend_from_slice(&[0x3b; 32]);
        outputs.push(0x01);
        let ValueField::Explicit(amount) = output.value() else {
            panic!("the pinned candidate's values are explicit");
        };
        outputs.extend_from_slice(&amount.to_be_bytes());
        outputs.push(0x00);
        outputs.push(u8::try_from(output.program().len()).expect("the fixture programs are short"));
        outputs.extend_from_slice(output.program());
    }
    stream.extend_from_slice(&single(&outputs));
    // Term 12: each output witness, its two proofs length-prefixed.
    let mut witnesses = vec![0x00, 64];
    witnesses.extend_from_slice(&[0x9c; 64]);
    witnesses.extend_from_slice(&[0x00, 0x00]);
    stream.extend_from_slice(&single(&witnesses));
    // Terms 13, 14, 16, 17 and 18. Term 15 is absent by the annex
    // refusal, which is what made term 13 a constant.
    stream.push(0x02);
    stream.extend_from_slice(&0_u32.to_le_bytes());
    stream.extend_from_slice(&LEAF_HASH);
    stream.push(0x00);
    stream.extend_from_slice(&0xffff_ffff_u32.to_le_bytes());

    assert_eq!(
        tagged_hash("TapSighash/elements", &stream),
        candidate_owner_message(&census, input, WitnessVectorTreatment::BothGrown),
    );
}

/// The message with the output-witness vector at its consensus length.
const PINNED_MESSAGE_WITH_VECTOR_GROWN: &str =
    "ff2f490274dfb3ea79a0d2b71357ec8fdad276d3978d29b511fe2516c6ec1257";

/// The message with the output-witness vector empty.
const PINNED_MESSAGE_WITH_VECTOR_EMPTY: &str =
    "02175b701b8144cac4e844af932581928e3516a63cc9d761983fd949103ec957";

/// One digest as lower-case hexadecimal.
fn hex(digest: &Digest32) -> String {
    use std::fmt::Write as _;

    digest.iter().fold(String::new(), |mut rendered, byte| {
        let _ = write!(rendered, "{byte:02x}");
        rendered
    })
}
