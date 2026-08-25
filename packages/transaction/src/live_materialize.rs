//! The transaction-wide confidential materializer, and the only
//! complete-private construction entry point there is.
//!
//! # Why the per-output role could not be extended
//!
//! [`crate::live_private::PrivateValueCapability`] cannot see a
//! predecessor opening, cannot balance across destinations, returns only
//! a commitment, fixes a null nonce, and cannot carry a proof. Those five
//! limits are not gaps to be filled one at a time: a transaction-wide
//! balance problem cannot be solved on a subset of its own terms, so a
//! per-output interface is the wrong shape for the question rather than a
//! smaller answer to it. The role stays for the per-output path it
//! already serves, and a request that reaches this function carrying it
//! is [`MaterializationRefusal::PerOutputMaterializationRefused`].
//!
//! # Why this component exists at all
//!
//! The target's own blinding machinery is not used to produce this
//! workspace's confidential outputs, and that is an election rather
//! than a limitation found late
//! `(´[PLAN-rule:exclusions:wallet-blinding]´)`. No reviewed stock RPC
//! produces the selected form, and the three obvious workarounds —
//! calling the raw blinding RPC, retrying ordinary wallet blinding
//! until an output happens to look useful, and mutating a fully
//! blinded asset back to explicit after proof construction — are
//! rejected by the charter. This module is what that rejection
//! required to be built, so its existence is the row that enforces
//! the exclusion.
//!
//! # The cryptography is injected, and the injection is the point
//!
//! This crate may not depend on the conformance package: that package is
//! the independent oracle this crate's output is compared against, and
//! the absence of the edge is what keeps the comparison from being one
//! opinion wearing two hats. So the two cryptographic collaborators are
//! TRAITS declared here and implemented outside, exactly as
//! `PrivateValueCapability` and `LiveCurveCapability` already are, and
//! the implementations live in the one library that can see both sides.
//!
//! [`ConfidentialProofMaterializer`] produces the construction's own
//! answers. [`IndependentCommitmentCheck`] recomputes them from a
//! different origin. Their two answers travel in two DIFFERENT types, so
//! [`commitments_agree`] cannot be handed the same origin twice — the
//! property is carried by the type system and not by a reviewer noticing.
//!
//! # Nothing here is a target verdict
//!
//! Every refusal in this module is a CONSTRUCTION refusal. None of them
//! says anything about a chain, none may be re-reported as a target
//! outcome, and none carries a funded observation. A fixture lookup that
//! failed is not evidence about a target.
//!
//! # No opening leaves, and no subtotal either
//!
//! No member of the result exposes an opening, a blinder, a nonce input,
//! or a proof input, and no refusal names a subtotal, an amount, or an
//! opening — a refusal that leaked one would have published exactly what
//! keeps amounts off the funding wire in the first place.
//!
//! # The owner sighash is somebody else's
//!
//! This module freezes a candidate and stops. It computes no digest,
//! selects no signing profile, and accepts neither. What the protected
//! bytes are hashed with, and by whom, belongs to the separately reviewed
//! owner-sighash work.

use std::collections::{BTreeMap, BTreeSet};

use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements::ReproducibilityContract;

use crate::bytes::{
    AssetField, AssetId, COMMITMENT_BYTES, InputWitness, NonceField, Outpoint, OutputWitness,
    TargetInput, TargetOutput, TargetTransaction, ValueField,
};
use crate::live_finalize::protected_preimage;

/// How many bytes a derived scalar occupies.
pub const SCALAR_BYTES: usize = 32;

/// How many bytes a fixture drift digest occupies.
pub const FIXTURE_DIGEST_BYTES: usize = 32;

// --- Origins ------------------------------------------------------------

/// Which of the four origins a commitment came from.
///
/// The workspace's own classification, carried here as a value so that a
/// comparison can check it rather than a reader having to know which
/// implementation was wired in. The table it reproduces is the one the
/// conformance package's manifest already argues at length: adopting the
/// same zero-knowledge bindings the target vendors would make an oracle
/// and the thing it checks one opinion wearing two hats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CommitmentOrigin {
    /// The construction's own output. Never its own expectation.
    ConstructionMaterializer,
    /// First-party bignum arithmetic over published constants.
    /// Independence unqualified.
    FirstPartyBignumOracle,
    /// The target's own implementation, through its own bindings.
    /// Conformance evidence, and not independence.
    ReferenceImplementation,
    /// The target's observation, read back from mined bytes.
    TargetReadback,
}

impl CommitmentOrigin {
    /// The complete census, in the table's own order.
    pub const ALL: &'static [Self] = &[
        Self::ConstructionMaterializer,
        Self::FirstPartyBignumOracle,
        Self::ReferenceImplementation,
        Self::TargetReadback,
    ];

    /// Whether this origin may serve as the independent check on a
    /// construction.
    ///
    /// The reference implementation may NOT, and that is the whole
    /// reason this predicate exists rather than an inequality: it binds
    /// the same library the target vendors, so its agreement with a
    /// materializer built on the same library is conformance evidence
    /// wearing independence's name. A readback is a target observation
    /// and is not available before submission, which leaves exactly one
    /// admissible answer.
    #[must_use]
    pub const fn may_check_a_construction(self) -> bool {
        matches!(self, Self::FirstPartyBignumOracle)
    }
}

/// A value commitment the construction's own materializer produced.
///
/// Opaque, and its constructor is named for its origin rather than for
/// its bytes. The implementation legitimately lives outside this crate,
/// so the constructor is reachable from outside it; what the type carries
/// is that a value of it cannot be mistaken for, or compared against, a
/// value of the other origin's type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterializerCommitment([u8; COMMITMENT_BYTES]);

impl MaterializerCommitment {
    /// The commitment a proof materializer computed.
    ///
    /// Called only by an implementation of
    /// [`ConfidentialProofMaterializer`]. Calling it anywhere else is
    /// mislabelling an origin, which no signature can prevent and which
    /// the naming is chosen to make obvious in review.
    #[must_use]
    pub const fn from_materializer(bytes: [u8; COMMITMENT_BYTES]) -> Self {
        Self(bytes)
    }

    /// The serialized commitment.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; COMMITMENT_BYTES] {
        &self.0
    }

    /// The commitment's prefix byte, which carries its parity.
    #[must_use]
    pub const fn prefix(&self) -> u8 {
        self.0[0]
    }
}

/// A value commitment an independent check recomputed.
///
/// The sibling of [`MaterializerCommitment`], and deliberately not the
/// same type: a function taking one of each cannot be handed two
/// materialized values, two recomputations, or one value twice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndependentCommitment([u8; COMMITMENT_BYTES]);

impl IndependentCommitment {
    /// The commitment an independent recomputation produced.
    #[must_use]
    pub const fn from_independent_recomputation(bytes: [u8; COMMITMENT_BYTES]) -> Self {
        Self(bytes)
    }

    /// The serialized commitment.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; COMMITMENT_BYTES] {
        &self.0
    }
}

/// Whether two origins' answers are the same bytes.
///
/// The only comparison this module admits, and it takes one of each type
/// on purpose. There is no overload of it that compares a materialized
/// value with a materialized value, because such a comparison is not
/// evidence and the way to keep it from being written is to make it
/// unspellable.
#[must_use]
pub fn commitments_agree(
    materialized: &MaterializerCommitment,
    independent: &IndependentCommitment,
) -> bool {
    materialized.bytes() == independent.bytes()
}

// --- The injected collaborators ----------------------------------------

/// One range-proof request, whole.
///
/// A parameter object rather than seven arguments, because six of them
/// are byte strings and an implementation that transposed two would
/// produce a proof about the wrong thing and no signature would notice.
#[derive(Clone, Copy, Debug)]
pub struct RangeproofRequest<'material> {
    output: usize,
    value_commitment: &'material MaterializerCommitment,
    explicit_asset: AssetId,
    semantic_amount: u64,
    value_blinder: &'material [u8; SCALAR_BYTES],
    seed: &'material [u8; SCALAR_BYTES],
    output_program: &'material [u8],
}

impl<'material> RangeproofRequest<'material> {
    /// The request for one output's proof.
    #[must_use]
    pub const fn new(
        output: usize,
        value_commitment: &'material MaterializerCommitment,
        explicit_asset: AssetId,
        semantic_amount: u64,
        value_blinder: &'material [u8; SCALAR_BYTES],
        seed: &'material [u8; SCALAR_BYTES],
        output_program: &'material [u8],
    ) -> Self {
        Self {
            output,
            value_commitment,
            explicit_asset,
            semantic_amount,
            value_blinder,
            seed,
            output_program,
        }
    }

    /// Which output the proof is for.
    #[must_use]
    pub const fn output(&self) -> usize {
        self.output
    }

    /// The commitment the proof must be bound to.
    #[must_use]
    pub const fn value_commitment(&self) -> &MaterializerCommitment {
        self.value_commitment
    }

    /// The explicit asset whose unblinded generator the proof uses.
    #[must_use]
    pub const fn explicit_asset(&self) -> AssetId {
        self.explicit_asset
    }

    /// The semantic amount the commitment carries.
    #[must_use]
    pub const fn semantic_amount(&self) -> u64 {
        self.semantic_amount
    }

    /// The value blinder the commitment opens with.
    #[must_use]
    pub const fn value_blinder(&self) -> &[u8; SCALAR_BYTES] {
        self.value_blinder
    }

    /// The deterministic seed the proof is generated from.
    #[must_use]
    pub const fn seed(&self) -> &[u8; SCALAR_BYTES] {
        self.seed
    }

    /// The output program the proof commits to.
    #[must_use]
    pub const fn output_program(&self) -> &[u8] {
        self.output_program
    }
}

/// What a materializer says its proof is bound to.
///
/// Declared by the implementation and checked against what was asked,
/// which is what makes a proof about the wrong commitment a refusal here
/// rather than a rejection at the target three steps later.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializedRangeproof {
    proof: Vec<u8>,
    surjection_proof: Vec<u8>,
    bound_commitment: [u8; COMMITMENT_BYTES],
    bound_asset: AssetId,
    bound_program: Vec<u8>,
}

impl MaterializedRangeproof {
    /// The proof a materializer produced, with what it says it proves.
    #[must_use]
    pub const fn new(
        proof: Vec<u8>,
        surjection_proof: Vec<u8>,
        bound_commitment: [u8; COMMITMENT_BYTES],
        bound_asset: AssetId,
        bound_program: Vec<u8>,
    ) -> Self {
        Self {
            proof,
            surjection_proof,
            bound_commitment,
            bound_asset,
            bound_program,
        }
    }

    /// The proof bytes.
    #[must_use]
    pub fn proof(&self) -> &[u8] {
        &self.proof
    }

    /// The surjection proof, which the hybrid form requires to be empty.
    #[must_use]
    pub fn surjection_proof(&self) -> &[u8] {
        &self.surjection_proof
    }

    /// Whether this proof is bound to what the request asked about.
    #[must_use]
    pub fn binds(&self, request: &RangeproofRequest<'_>) -> bool {
        self.bound_commitment == *request.value_commitment().bytes()
            && self.bound_asset == request.explicit_asset()
            && self.bound_program == request.output_program()
    }
}

/// The confidential cryptography a transaction-wide construction needs.
///
/// Every method may answer `None`, and every `None` is an ordinary
/// construction refusal rather than an error condition: under byte
/// identity there is exactly one deterministic answer, and if it does not
/// work the ceremony refuses rather than searching until it does. No
/// method returns an opening, and none takes one back out.
pub trait ConfidentialProofMaterializer {
    /// Which origin this implementation's commitments have.
    fn origin(&self) -> CommitmentOrigin;

    /// The value commitment one output's amount and blinder produce.
    fn value_commitment(
        &self,
        explicit_asset: AssetId,
        semantic_amount: u64,
        value_blinder: &[u8; SCALAR_BYTES],
    ) -> Option<MaterializerCommitment>;

    /// The nonce field one derived nonce input produces.
    fn nonce_commitment(&self, nonce_input: &[u8; SCALAR_BYTES]) -> Option<[u8; COMMITMENT_BYTES]>;

    /// The range proof for one output.
    ///
    /// A refusal here adds no randomness and triggers no retry. There is
    /// one deterministic answer and a failure to produce it is a refusal.
    fn range_proof(&self, request: &RangeproofRequest<'_>) -> Option<MaterializedRangeproof>;
}

/// The independent recomputation a construction is checked against.
///
/// Its only admitted implementation is an adapter over the first-party
/// bignum commitment oracle. The reference bindings may implement
/// [`ConfidentialProofMaterializer`] and may NOT implement this, because
/// they bind the same library the target vendors and their agreement
/// would be conformance evidence rather than independence.
pub trait IndependentCommitmentCheck {
    /// Which origin this implementation's commitments have.
    fn origin(&self) -> CommitmentOrigin;

    /// The commitment this origin computes for the same opening.
    fn recompute(
        &self,
        explicit_asset: AssetId,
        semantic_amount: u64,
        value_blinder: &[u8; SCALAR_BYTES],
    ) -> Option<IndependentCommitment>;

    /// The balancing blinder, solved as the input blinder sum minus the
    /// other outputs' sum in the group.
    ///
    /// `None` for a solved scalar that is zero or out of range. The
    /// arithmetic lives with the independent origin because this crate
    /// carries no bignum dependency and may not acquire one: reaching for
    /// a new cryptographic crate would be leaving the charter.
    fn solve_balancing_blinder(
        &self,
        input_blinder_sum: &[u8; SCALAR_BYTES],
        other_blinders: &[[u8; SCALAR_BYTES]],
    ) -> Option<[u8; SCALAR_BYTES]>;
}

// --- Profiles -----------------------------------------------------------

/// Which custody model the ceremony runs under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfidentialCustodyProfile {
    /// One party holds every opening and every opening is published.
    CentralPublicFixtures,
}

/// Which materializer the ceremony selects.
///
/// The second member is the retired per-output role, named here so that
/// selecting it is a typed refusal rather than a silent substitution. A
/// vocabulary that could not name the wrong answer could not refuse it
/// either.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfidentialMaterializerProfile {
    /// The transaction-wide deterministic materializer.
    GuideCtfDeterministicV1,
    /// The per-output confidential value role.
    PerOutputValueCapability,
}

/// Which proof form the outputs carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfidentialProofProfile {
    /// A range proof per confidential value and an empty surjection
    /// field, which is the explicit-asset confidential-value form.
    ExplicitAssetRangeproofV1,
}

/// Where nonce material comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfidentialNonceProfile {
    /// Derived deterministically, per output, from the case identity.
    DeterministicDerivedV1,
}

/// What fixes the output order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfidentialOrderProfile {
    /// The fixture's own order, and never a commitment prefix, an
    /// amount, or the order a retry happened to finish in.
    FixtureFixedOrder,
}

/// What happens after a refusal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfidentialRetryProfile {
    /// Nothing. A proof failure is never a source of entropy.
    NoRetry,
}

/// The seven profiles one materialization runs under.
///
/// Selected before construction and carried unchanged. An unsupported
/// combination refuses rather than defaulting to a supported neighbour,
/// because a run that quietly moved to a neighbouring profile would
/// produce a report naming a profile it did not use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfidentialMaterializationProfiles {
    /// Which reproducibility contract the run claims.
    pub reproducibility_contract: ReproducibilityContract,
    /// Who holds the openings.
    pub custody_profile: ConfidentialCustodyProfile,
    /// Which materializer builds the fields.
    pub materializer_profile: ConfidentialMaterializerProfile,
    /// Which proof form the outputs carry.
    pub proof_profile: ConfidentialProofProfile,
    /// Where nonce material comes from.
    pub nonce_profile: ConfidentialNonceProfile,
    /// What fixes the output order.
    pub order_profile: ConfidentialOrderProfile,
    /// What happens after a refusal.
    pub retry_profile: ConfidentialRetryProfile,
}

impl ConfidentialMaterializationProfiles {
    /// Whether this build supports every selected profile AND the
    /// combination.
    ///
    /// Two questions and not one: a build may support each of two
    /// profiles and support neither of them together, and answering the
    /// combination by answering the members would be the defaulting the
    /// rule forbids.
    #[must_use]
    pub const fn supported(&self) -> bool {
        if !matches!(
            self.materializer_profile,
            ConfidentialMaterializerProfile::GuideCtfDeterministicV1
        ) {
            return false;
        }
        matches!(
            self.reproducibility_contract,
            ReproducibilityContract::ByteIdentity
        )
    }
}

// --- The frozen view ----------------------------------------------------

/// Which role one fixture output plays.
///
/// Mirrors the fixture registry's roles. It is a second spelling only in
/// the sense that two packages with no edge between them must each be
/// able to say the word: this crate may not depend on the one where the
/// registry lives, and a view it cannot name is not a view.
///
/// # It is NARROWER than the registry's vocabulary, deliberately
///
/// The registry distinguishes an output solved from the others from the
/// sole output of a single-output manifest. Both project onto
/// [`Self::Balancing`] here, and nothing is lost: the view's balancing
/// role means "this blinder is solved from the others", and the sole form
/// is that instruction with no others — the solve over an empty set
/// returns the input blinder sum, which is exactly what the sole form
/// wants. What the registry's extra member carries is a DECLARATION about
/// a manifest's shape, and a declaration has done its work by the time
/// the manifest is registered.
///
/// The registry's FEE role now has a member here too, and the order in
/// which that happened is the point. The member was withheld while the
/// stages below could not build an output with an explicit value, no
/// nonce and no range proof, because a member without those stages would
/// have mapped a fee onto the committed path and produced a BLINDED fee
/// output — which the target does not recognize as a fee at all, and
/// which would have been a silently wrong transaction rather than an
/// honest stop. The member arrives with the stages, not before them.
///
/// What the member costs the vocabulary is the assumption that every
/// output carries an opening. It does not: a fee output's blinder,
/// nonce and range-proof seed are ABSENT rather than zero, which is why
/// [`ConfidentialFixtureOutputView`]'s three scalar accessors return an
/// option. Zero-filling them would have made a record of zeroes read
/// like an opening, and an explicit output has none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfidentialOutputRole {
    /// An ordinary output whose blinder is derived.
    Primary,
    /// The one output per transaction whose blinder is solved.
    Balancing,
    /// The mandatorily explicit fee output: explicit value, explicit
    /// asset, empty program, no opening, and no witness.
    ///
    /// It is outside the blinder solve at a zero blinder, which is not a
    /// convention here but the target's arithmetic: an explicit value is
    /// committed with an all-zero blinder and joins the same Pedersen
    /// tally, so a fee can never be the output that absorbs the input
    /// blinder sum.
    Fee,
}

/// Which derivation role a scalar was derived for.
///
/// The same three the derivation recipe separates by, for the same
/// placement reason as [`ConfidentialOutputRole`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DerivationRole {
    /// An output's value blinder.
    ValueBlinder,
    /// An output's nonce secret.
    NonceSecret,
    /// An output's range-proof seed.
    RangeproofSeed,
}

/// What the bounded parity search concluded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParityOutcome {
    /// The required prefixes were found, at this counter.
    Settled {
        /// Which counter settled it.
        counter: u16,
    },
    /// The counter reached its bound without the required prefix pair.
    Exhausted {
        /// How many counters were tried.
        attempts: u32,
    },
}

/// One fixture output, as the view projects it.
///
/// It carries the opening because the materializer needs it to build the
/// commitment, and it carries no registration and no freezing because
/// this is a projection of a frozen registry rather than a second one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfidentialFixtureOutputView {
    role: ConfidentialOutputRole,
    semantic_amount: u64,
    output_program: Vec<u8>,
    value_blinder: Option<[u8; SCALAR_BYTES]>,
    nonce_input: Option<[u8; SCALAR_BYTES]>,
    rangeproof_seed: Option<[u8; SCALAR_BYTES]>,
}

impl ConfidentialFixtureOutputView {
    /// One projected fixture output that carries an opening.
    ///
    /// Every committed role is built here. A fee output cannot be: it has
    /// no opening to pass, which is what [`Self::fee`] exists to say.
    #[must_use]
    pub const fn new(
        role: ConfidentialOutputRole,
        semantic_amount: u64,
        output_program: Vec<u8>,
        value_blinder: [u8; SCALAR_BYTES],
        nonce_input: [u8; SCALAR_BYTES],
        rangeproof_seed: [u8; SCALAR_BYTES],
    ) -> Self {
        Self {
            role,
            semantic_amount,
            output_program,
            value_blinder: Some(value_blinder),
            nonce_input: Some(nonce_input),
            rangeproof_seed: Some(rangeproof_seed),
        }
    }

    /// One projected fee output.
    ///
    /// It takes an amount and nothing else. The empty program is not a
    /// parameter because it is not a choice — an output with a program is
    /// not a fee at the target — and the three opening scalars are absent
    /// rather than zero, so nothing downstream can mistake a placeholder
    /// for an opening.
    #[must_use]
    pub const fn fee(semantic_amount: u64) -> Self {
        Self {
            role: ConfidentialOutputRole::Fee,
            semantic_amount,
            output_program: Vec::new(),
            value_blinder: None,
            nonce_input: None,
            rangeproof_seed: None,
        }
    }

    /// Which role this output plays.
    #[must_use]
    pub const fn role(&self) -> ConfidentialOutputRole {
        self.role
    }

    /// The semantic amount.
    #[must_use]
    pub const fn semantic_amount(&self) -> u64 {
        self.semantic_amount
    }

    /// The output program.
    #[must_use]
    pub fn output_program(&self) -> &[u8] {
        &self.output_program
    }

    /// The value blinder, where this output has one.
    ///
    /// Absent for a fee output, whose value is explicit.
    #[must_use]
    pub const fn value_blinder(&self) -> Option<&[u8; SCALAR_BYTES]> {
        self.value_blinder.as_ref()
    }

    /// The nonce input, where this output has one.
    #[must_use]
    pub const fn nonce_input(&self) -> Option<&[u8; SCALAR_BYTES]> {
        self.nonce_input.as_ref()
    }

    /// The range-proof seed, where this output has one.
    #[must_use]
    pub const fn rangeproof_seed(&self) -> Option<&[u8; SCALAR_BYTES]> {
        self.rangeproof_seed.as_ref()
    }
}

/// One resolved fixture, as the view projects it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfidentialFixtureView {
    digest: [u8; FIXTURE_DIGEST_BYTES],
    explicit_asset: AssetId,
    input_blinder_sum: [u8; SCALAR_BYTES],
    parity: ParityOutcome,
    outputs: Vec<ConfidentialFixtureOutputView>,
}

impl ConfidentialFixtureView {
    /// One projected fixture.
    #[must_use]
    pub const fn new(
        digest: [u8; FIXTURE_DIGEST_BYTES],
        explicit_asset: AssetId,
        input_blinder_sum: [u8; SCALAR_BYTES],
        parity: ParityOutcome,
        outputs: Vec<ConfidentialFixtureOutputView>,
    ) -> Self {
        Self {
            digest,
            explicit_asset,
            input_blinder_sum,
            parity,
            outputs,
        }
    }

    /// The drift digest this fixture registered under.
    #[must_use]
    pub const fn digest(&self) -> &[u8; FIXTURE_DIGEST_BYTES] {
        &self.digest
    }

    /// The explicit protocol asset.
    #[must_use]
    pub const fn explicit_asset(&self) -> AssetId {
        self.explicit_asset
    }

    /// The confidential input blinder sum the outputs balance against.
    #[must_use]
    pub const fn input_blinder_sum(&self) -> &[u8; SCALAR_BYTES] {
        &self.input_blinder_sum
    }

    /// What the bounded parity search concluded.
    #[must_use]
    pub const fn parity(&self) -> ParityOutcome {
        self.parity
    }

    /// Every projected output, in the fixture's own fixed order.
    #[must_use]
    pub fn outputs(&self) -> &[ConfidentialFixtureOutputView] {
        &self.outputs
    }
}

/// The narrow read-only projection of a frozen fixture registry.
///
/// It supplies roles, amounts, programs, asset, fixed order, the
/// balancing designation, and openings. It supplies no registration and
/// no freezing, because those belong to the registry and a projection
/// that offered them would be a second registry with a different name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FrozenConfidentialFixtureView {
    entries: BTreeMap<String, ConfidentialFixtureView>,
}

impl FrozenConfidentialFixtureView {
    /// The projection over these fixtures.
    #[must_use]
    pub const fn new(entries: BTreeMap<String, ConfidentialFixtureView>) -> Self {
        Self { entries }
    }

    /// The fixture this handle names, if the view holds one.
    #[must_use]
    pub fn fixture(&self, handle: &str) -> Option<&ConfidentialFixtureView> {
        self.entries.get(handle)
    }

    /// Every handle the view holds.
    #[must_use]
    pub fn handles(&self) -> Vec<&str> {
        self.entries.keys().map(String::as_str).collect()
    }
}

// --- The intent ---------------------------------------------------------

/// One reference into the frozen view.
///
/// Handle, digest, and output index together, because a handle alone is
/// an identity and a digest alone is a drift check, and binding to one
/// without the other is binding to half of what was registered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureOpeningReference {
    handle: String,
    digest: [u8; FIXTURE_DIGEST_BYTES],
    output: usize,
}

impl FixtureOpeningReference {
    /// The reference naming one output of one registered fixture.
    #[must_use]
    pub const fn new(handle: String, digest: [u8; FIXTURE_DIGEST_BYTES], output: usize) -> Self {
        Self {
            handle,
            digest,
            output,
        }
    }

    /// The public fixture handle.
    #[must_use]
    pub fn handle(&self) -> &str {
        &self.handle
    }

    /// The drift digest.
    #[must_use]
    pub const fn digest(&self) -> &[u8; FIXTURE_DIGEST_BYTES] {
        &self.digest
    }

    /// Which output of that fixture.
    #[must_use]
    pub const fn output(&self) -> usize {
        self.output
    }
}

/// One confidential input the candidate consumes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfidentialInputIntent {
    outpoint: Outpoint,
    observed_asset: AssetField,
    observed_value: ValueField,
    observed_program: Vec<u8>,
    sequence: u32,
    opening: FixtureOpeningReference,
    explicit_amount: u64,
    zero_asset_blinder: [u8; SCALAR_BYTES],
}

impl ConfidentialInputIntent {
    /// One consumed confidential predecessor output.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        outpoint: Outpoint,
        observed_asset: AssetField,
        observed_value: ValueField,
        observed_program: Vec<u8>,
        sequence: u32,
        opening: FixtureOpeningReference,
        explicit_amount: u64,
        zero_asset_blinder: [u8; SCALAR_BYTES],
    ) -> Self {
        Self {
            outpoint,
            observed_asset,
            observed_value,
            observed_program,
            sequence,
            opening,
            explicit_amount,
            zero_asset_blinder,
        }
    }

    /// The outpoint spent.
    #[must_use]
    pub const fn outpoint(&self) -> Outpoint {
        self.outpoint
    }

    /// The asset field the target was observed to hold.
    #[must_use]
    pub const fn observed_asset(&self) -> AssetField {
        self.observed_asset
    }

    /// The value field the target was observed to hold.
    #[must_use]
    pub const fn observed_value(&self) -> ValueField {
        self.observed_value
    }

    /// The program the target was observed to hold.
    #[must_use]
    pub fn observed_program(&self) -> &[u8] {
        &self.observed_program
    }

    /// The sequence field the spend carries.
    #[must_use]
    pub const fn sequence(&self) -> u32 {
        self.sequence
    }

    /// Where the opening lives.
    #[must_use]
    pub const fn opening(&self) -> &FixtureOpeningReference {
        &self.opening
    }

    /// The semantic amount the opening carries.
    #[must_use]
    pub const fn explicit_amount(&self) -> u64 {
        self.explicit_amount
    }

    /// The asset blinder, which the protocol region fixes at zero.
    #[must_use]
    pub const fn zero_asset_blinder(&self) -> &[u8; SCALAR_BYTES] {
        &self.zero_asset_blinder
    }
}

/// One confidential destination the candidate creates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfidentialDestinationIntent {
    semantic_amount: u64,
    explicit_asset: AssetId,
    output_program: Vec<u8>,
    fixture: FixtureOpeningReference,
    role: ConfidentialOutputRole,
}

impl ConfidentialDestinationIntent {
    /// One created confidential output.
    #[must_use]
    pub const fn new(
        semantic_amount: u64,
        explicit_asset: AssetId,
        output_program: Vec<u8>,
        fixture: FixtureOpeningReference,
        role: ConfidentialOutputRole,
    ) -> Self {
        Self {
            semantic_amount,
            explicit_asset,
            output_program,
            fixture,
            role,
        }
    }

    /// The semantic amount.
    #[must_use]
    pub const fn semantic_amount(&self) -> u64 {
        self.semantic_amount
    }

    /// The explicit protocol asset.
    #[must_use]
    pub const fn explicit_asset(&self) -> AssetId {
        self.explicit_asset
    }

    /// The output program.
    #[must_use]
    pub fn output_program(&self) -> &[u8] {
        &self.output_program
    }

    /// Which fixture output this destination realizes.
    #[must_use]
    pub const fn fixture(&self) -> &FixtureOpeningReference {
        &self.fixture
    }

    /// Which role it plays in the balance.
    #[must_use]
    pub const fn role(&self) -> ConfidentialOutputRole {
        self.role
    }
}

/// One member of the non-protocol region.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NonProtocolMember {
    asset: AssetId,
    amount: u64,
    output_program: Vec<u8>,
}

impl NonProtocolMember {
    /// One explicit non-protocol output.
    #[must_use]
    pub const fn new(asset: AssetId, amount: u64, output_program: Vec<u8>) -> Self {
        Self {
            asset,
            amount,
            output_program,
        }
    }

    /// Its explicit asset.
    #[must_use]
    pub const fn asset(&self) -> AssetId {
        self.asset
    }

    /// Its explicit amount.
    #[must_use]
    pub const fn amount(&self) -> u64 {
        self.amount
    }

    /// Its output program.
    #[must_use]
    pub fn output_program(&self) -> &[u8] {
        &self.output_program
    }
}

/// The region the protocol balance excludes.
///
/// Excluded from BOTH equations, semantic and blinder. A policy fee and a
/// policy change output are not protocol members and counting them in
/// either would turn the balance into an argument about which region a
/// member was put in.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NonProtocolFundingRegion {
    members: Vec<NonProtocolMember>,
}

impl NonProtocolFundingRegion {
    /// The region carrying these members.
    #[must_use]
    pub const fn new(members: Vec<NonProtocolMember>) -> Self {
        Self { members }
    }

    /// Every member.
    #[must_use]
    pub fn members(&self) -> &[NonProtocolMember] {
        &self.members
    }
}

/// Everything one transaction-wide materialization is asked to build.
///
/// Consumed whole. There is no builder that can be half-applied and no
/// stage that may be called on a partial intent, because a
/// transaction-wide balance problem cannot be solved on a subset of its
/// own terms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfidentialConstructionIntent {
    inputs: Vec<ConfidentialInputIntent>,
    destinations: Vec<ConfidentialDestinationIntent>,
    non_protocol_region: NonProtocolFundingRegion,
    profiles: ConfidentialMaterializationProfiles,
    version: u32,
    lock_time: u32,
}

impl ConfidentialConstructionIntent {
    /// One whole intent.
    #[must_use]
    pub const fn new(
        inputs: Vec<ConfidentialInputIntent>,
        destinations: Vec<ConfidentialDestinationIntent>,
        non_protocol_region: NonProtocolFundingRegion,
        profiles: ConfidentialMaterializationProfiles,
        version: u32,
        lock_time: u32,
    ) -> Self {
        Self {
            inputs,
            destinations,
            non_protocol_region,
            profiles,
            version,
            lock_time,
        }
    }

    /// Every consumed input.
    #[must_use]
    pub fn inputs(&self) -> &[ConfidentialInputIntent] {
        &self.inputs
    }

    /// Every created confidential destination.
    #[must_use]
    pub fn destinations(&self) -> &[ConfidentialDestinationIntent] {
        &self.destinations
    }

    /// The excluded region.
    #[must_use]
    pub const fn non_protocol_region(&self) -> &NonProtocolFundingRegion {
        &self.non_protocol_region
    }

    /// The selected profiles.
    #[must_use]
    pub const fn profiles(&self) -> ConfidentialMaterializationProfiles {
        self.profiles
    }

    /// The transaction version.
    #[must_use]
    pub const fn version(&self) -> u32 {
        self.version
    }

    /// The lock-time field.
    #[must_use]
    pub const fn lock_time(&self) -> u32 {
        self.lock_time
    }
}

// --- Refusals -----------------------------------------------------------

/// Which family member a refusal is about.
///
/// The family classification's own member identity, and not a second
/// vocabulary: a member is an input, a protocol output, or a non-protocol
/// output, and every classification here places every member in exactly
/// one of the three.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FamilyMember {
    /// One consumed input, by position.
    Input(usize),
    /// One created protocol output, by position.
    ProtocolOutput(usize),
    /// One created non-protocol output, by position.
    NonProtocolOutput(usize),
}

/// Which protected region a post-finalization mutation would touch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProofFinalizedRegion {
    /// The input census.
    Inputs,
    /// The output census, every field of it.
    Outputs,
    /// The nonce fields.
    NonceFields,
    /// The output-witness vector, and therefore every proof.
    OutputWitnesses,
    /// The version field.
    Version,
    /// The lock-time field.
    LockTime,
    /// The sponsor region, present or absent.
    SponsorRegion,
    /// The fee region, present or absent.
    FeeRegion,
}

impl ProofFinalizedRegion {
    /// The complete census, in the freeze rule's own order.
    pub const ALL: &'static [Self] = &[
        Self::Inputs,
        Self::Outputs,
        Self::NonceFields,
        Self::OutputWitnesses,
        Self::Version,
        Self::LockTime,
        Self::SponsorRegion,
        Self::FeeRegion,
    ];
}

/// Why a transaction-wide confidential materialization did not happen.
///
/// Closed, with no catch-all. Every variant is a CONSTRUCTION refusal:
/// none is a target verdict, none may be re-reported as one, and none
/// carries a funded observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaterializationRefusal {
    /// No opening in the view answers to this input's reference.
    PredecessorOpeningMissing {
        /// Which input.
        outpoint: Outpoint,
    },
    /// The opening does not recompute the commitment the target was
    /// observed to hold.
    PredecessorOpeningMismatch {
        /// Which input.
        outpoint: Outpoint,
    },
    /// The protocol region's semantic amounts do not close.
    SemanticValueImbalance,
    /// The protocol region's value blinders do not close.
    ValueBlinderImbalance,
    /// A scalar was zero, out of range, or unsolvable.
    InvalidScalar {
        /// Which derivation role it was for.
        role: DerivationRole,
    },
    /// A commitment was not computable, or landed on the identity.
    InvalidCommitment {
        /// Which output.
        output: usize,
    },
    /// The bounded parity search reached its bound.
    BoundedParitySearchExhausted,
    /// Nonce material was not producible for one output.
    NonceMaterializationFailed {
        /// Which output.
        output: usize,
    },
    /// A range proof was not producible for one output.
    ///
    /// No randomness is added and no retry follows.
    RangeproofMaterializationFailed {
        /// Which output.
        output: usize,
    },
    /// The frozen candidate's bytes do not decode and re-encode to
    /// themselves.
    SerializationRoundTripMismatch,
    /// Something tried to change a protected region after the freeze.
    PostFinalizationMutation {
        /// Which region.
        region: ProofFinalizedRegion,
    },
    /// The view holds no fixture under this handle.
    UnknownFixtureHandle {
        /// The offered handle.
        handle: String,
    },
    /// The view's fixture registered under a different digest.
    FixtureDigestMismatch {
        /// The offered handle.
        handle: String,
    },
    /// Two members bound to the same fixture output.
    FixtureBindingAmbiguous {
        /// The offered handle.
        handle: String,
    },
    /// The intent's order is not the fixture's own fixed order.
    FixtureOutputOrderMismatch,
    /// Two inputs spend the same previous output.
    DuplicateInputOutpoint {
        /// The repeated outpoint.
        outpoint: Outpoint,
    },
    /// A member was not placed in exactly one family.
    IncompleteFamilyClassification,
    /// A protocol member carries an asset that is not the protocol one.
    ProtocolAssetMismatch {
        /// Which member.
        member: FamilyMember,
    },
    /// A protocol member's asset field is a commitment.
    ///
    /// The one confidential form this workspace constructs pairs a
    /// committed value with an explicit asset
    /// `(´[PLAN-rule:exclusions:asset-blinding]´)`.
    ConfidentialProtocolAsset {
        /// Which member.
        member: FamilyMember,
    },
    /// A protocol member's asset blinder is not zero.
    NonzeroProtocolAssetBlinder {
        /// Which member.
        member: FamilyMember,
    },
    /// A non-protocol member occupies a protocol member's place.
    NonProtocolRegionOverlapsProtocol {
        /// Which member.
        member: FamilyMember,
    },
    /// A non-protocol member carries the protocol asset, so excluding it
    /// from the balance would be excluding protocol value.
    NonProtocolRegionAffectsProtocolBalance,
    /// This build does not support the selected profiles, or does not
    /// support them together.
    UnsupportedProfileCombination,
    /// The two origins disagreed about one commitment.
    IndependentCommitmentMismatch {
        /// Which output.
        output: usize,
    },
    /// The independent check is not an origin that may check this
    /// construction.
    IndependentCommitmentOriginNotDistinct {
        /// Which output.
        output: usize,
    },
    /// A proof is bound to something other than what was asked about.
    ProofBindingMismatch {
        /// Which output.
        output: usize,
    },
    /// A confidential output's range proof is empty.
    RangeproofEmpty {
        /// Which output.
        output: usize,
    },
    /// A materializer returned a surjection proof for a form that
    /// requires the field empty.
    ///
    /// The construction half of the elected asset-blinding exclusion
    /// `(´[PLAN-rule:exclusions:asset-blinding]´)`.
    UnexpectedSurjectionProof {
        /// Which output.
        output: usize,
    },
    /// The output-witness census is not one entry per output.
    OutputWitnessCensusMismatch,
    /// The opening-binding census is not one verified reference per
    /// confidential input.
    OpeningBindingCensusMismatch,
    /// A signer input would have carried an opening.
    SignerInputWouldExposeOpening {
        /// Which input.
        outpoint: Outpoint,
    },
    /// The request selected the retired per-output role.
    PerOutputMaterializationRefused,
    /// A fee output arrived carrying an output program.
    ///
    /// The target's own definition of a fee is an output with an EMPTY
    /// scriptPubKey, so an output with a program is not a fee however it
    /// is labelled. Refused rather than emptied, because emptying it
    /// would silently discard a destination somebody stated.
    FeeOutputProgramNotEmpty {
        /// Which output.
        output: usize,
    },
    /// A fee output arrived carrying an opening.
    ///
    /// A fee's value is explicit and an explicit value has no opening. A
    /// projection that supplied one has mapped some other role onto this
    /// one.
    FeeOutputCarriesAnOpening {
        /// Which output.
        output: usize,
    },
    /// A fee output would carry no value.
    ///
    /// A zero-value explicit output is admitted by the target only where
    /// its scriptPubKey is unspendable, and an empty script is not
    /// unspendable — so a zero-value fee output is refused outright,
    /// there rather than here. Refusing it here means the refusal names
    /// the fee rather than arriving as a target verdict on a shape this
    /// construction knew it could not build.
    FeeOutputValueZero {
        /// Which output.
        output: usize,
    },
    /// The emitted fee output does not satisfy the target's own fee
    /// predicate.
    ///
    /// The last clause of the fee stage, and the one that is not a
    /// restatement of the ones before it: the output is built and then
    /// asked whether it IS a fee, by the same three-conjunct predicate
    /// the target applies. A construction that believed it had built a
    /// fee and had not would stop here.
    FeeOutputNotRecognizable {
        /// Which output.
        output: usize,
    },
}

// --- The result ---------------------------------------------------------

/// One verified fixture reference, and no opening.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedFixtureReference {
    outpoint: Outpoint,
    handle: String,
    digest: [u8; FIXTURE_DIGEST_BYTES],
    output: usize,
}

impl VerifiedFixtureReference {
    /// Which input this reference belongs to.
    #[must_use]
    pub const fn outpoint(&self) -> Outpoint {
        self.outpoint
    }

    /// The public handle.
    #[must_use]
    pub fn handle(&self) -> &str {
        &self.handle
    }

    /// The drift digest.
    #[must_use]
    pub const fn digest(&self) -> &[u8; FIXTURE_DIGEST_BYTES] {
        &self.digest
    }

    /// Which output of that fixture.
    #[must_use]
    pub const fn output(&self) -> usize {
        self.output
    }
}

/// One verified fixture reference per confidential input, and no opening.
///
/// The census is what the result carries INSTEAD of the openings it was
/// built from. A reader who wants to check an opening resolves the
/// reference against the registry; the result does not carry one, so
/// there is no path by which reporting the result publishes one.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpeningBindingCensus {
    entries: Vec<VerifiedFixtureReference>,
}

impl OpeningBindingCensus {
    /// Every verified reference, in input order.
    #[must_use]
    pub fn entries(&self) -> &[VerifiedFixtureReference] {
        &self.entries
    }
}

/// One input a signer will be asked to authorize.
///
/// Observed target data and candidate structure. No opening, no blinder,
/// no nonce input, no key, and no proof input, and that exclusion is a
/// rule rather than an omission: the type that would have carried one is
/// never constructed at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofFinalizedSignerInput {
    outpoint: Outpoint,
    spent_asset: AssetField,
    spent_value: ValueField,
    spent_program: Vec<u8>,
    position: u16,
    byte_binding: Vec<u8>,
}

impl ProofFinalizedSignerInput {
    /// The outpoint spent.
    #[must_use]
    pub const fn outpoint(&self) -> Outpoint {
        self.outpoint
    }

    /// The spent output's asset field, as observed.
    #[must_use]
    pub const fn spent_asset(&self) -> AssetField {
        self.spent_asset
    }

    /// The spent output's value field, as observed.
    #[must_use]
    pub const fn spent_value(&self) -> ValueField {
        self.spent_value
    }

    /// The spent output's program, as observed.
    #[must_use]
    pub fn spent_program(&self) -> &[u8] {
        &self.spent_program
    }

    /// Which input position this is.
    #[must_use]
    pub const fn position(&self) -> u16 {
        self.position
    }

    /// The exact protected bytes this authorization is bound to.
    #[must_use]
    pub fn byte_binding(&self) -> &[u8] {
        &self.byte_binding
    }
}

/// One candidate whose proofs are final and whose protected bytes are
/// fixed.
///
/// Private fields, a constructor visible only to the materializer,
/// immutable accessors, and a region census. It EXTENDS the finalized
/// form rather than replacing or wrapping it: replacing would fork the
/// signing path for two representations that share every other protected
/// datum, and wrapping would leave an inner value's proof-free protected
/// bytes authoritative, which is the exact loss a wrapper would exist to
/// prevent. What it shares with the finalized form is the preimage rule
/// itself, computed by the same function, so the two cannot drift.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofFinalizedCandidate {
    protected: TargetTransaction,
    protected_bytes: Vec<u8>,
}

impl ProofFinalizedCandidate {
    /// Freezes one materialized transaction.
    ///
    /// The encoding law is checked here rather than trusted: the bytes
    /// are decoded and re-encoded, and a candidate whose bytes do not
    /// reproduce themselves is refused. Loss, an alternate encoding of
    /// the same value, wrong emptiness, and fragments all fail it.
    fn freeze(protected: TargetTransaction) -> Result<Self, MaterializationRefusal> {
        let bytes = protected.encode();
        let round_tripped = TargetTransaction::decode(&bytes)
            .map_err(|_| MaterializationRefusal::SerializationRoundTripMismatch)?;
        if round_tripped.encode() != bytes || round_tripped != protected {
            return Err(MaterializationRefusal::SerializationRoundTripMismatch);
        }
        let protected_bytes =
            protected_preimage(&protected, LiveTransferRepresentationPlan::PrivateCommitted);
        Ok(Self {
            protected,
            protected_bytes,
        })
    }

    /// The frozen transaction.
    #[must_use]
    pub const fn protected(&self) -> &TargetTransaction {
        &self.protected
    }

    /// The exact protected preimage every owner binds to.
    ///
    /// It contains the output-witness vector, so changing one proof byte
    /// changes it. That is the whole reason the private lane's preimage
    /// differs from the explicit lane's.
    #[must_use]
    pub fn protected_bytes(&self) -> &[u8] {
        &self.protected_bytes
    }

    /// Every region this candidate protects.
    ///
    /// All eight, for every candidate. A value of this type that
    /// protected seven would be the thing the freeze rule forbids, so the
    /// method reports the census rather than a subset of it.
    #[must_use]
    pub fn regions(&self) -> BTreeSet<ProofFinalizedRegion> {
        ProofFinalizedRegion::ALL.iter().copied().collect()
    }

    /// Inserting into a protected region.
    ///
    /// # Errors
    ///
    /// Always [`MaterializationRefusal::PostFinalizationMutation`]. The
    /// operation is spelled so that a caller reaching for it gets a
    /// refusal naming the region rather than a compile error naming
    /// nothing, and so that a test can walk the census.
    pub const fn insert(
        &self,
        region: ProofFinalizedRegion,
    ) -> Result<Self, MaterializationRefusal> {
        let _ = self;
        Err(MaterializationRefusal::PostFinalizationMutation { region })
    }

    /// Removing from a protected region.
    ///
    /// # Errors
    ///
    /// Always [`MaterializationRefusal::PostFinalizationMutation`].
    pub const fn remove(
        &self,
        region: ProofFinalizedRegion,
    ) -> Result<Self, MaterializationRefusal> {
        let _ = self;
        Err(MaterializationRefusal::PostFinalizationMutation { region })
    }

    /// Replacing part of a protected region.
    ///
    /// # Errors
    ///
    /// Always [`MaterializationRefusal::PostFinalizationMutation`].
    pub const fn replace(
        &self,
        region: ProofFinalizedRegion,
    ) -> Result<Self, MaterializationRefusal> {
        let _ = self;
        Err(MaterializationRefusal::PostFinalizationMutation { region })
    }

    /// Reordering a protected region.
    ///
    /// # Errors
    ///
    /// Always [`MaterializationRefusal::PostFinalizationMutation`].
    pub const fn reorder(
        &self,
        region: ProofFinalizedRegion,
    ) -> Result<Self, MaterializationRefusal> {
        let _ = self;
        Err(MaterializationRefusal::PostFinalizationMutation { region })
    }

    /// Regenerating the proofs.
    ///
    /// # Errors
    ///
    /// Always [`MaterializationRefusal::PostFinalizationMutation`] naming
    /// the output-witness region.
    pub const fn regenerate_proofs(&self) -> Result<Self, MaterializationRefusal> {
        let _ = self;
        Err(MaterializationRefusal::PostFinalizationMutation {
            region: ProofFinalizedRegion::OutputWitnesses,
        })
    }

    /// Repairing the proofs.
    ///
    /// # Errors
    ///
    /// Always [`MaterializationRefusal::PostFinalizationMutation`] naming
    /// the output-witness region.
    pub const fn repair_proofs(&self) -> Result<Self, MaterializationRefusal> {
        let _ = self;
        Err(MaterializationRefusal::PostFinalizationMutation {
            region: ProofFinalizedRegion::OutputWitnesses,
        })
    }

    /// Reblinding the outputs.
    ///
    /// # Errors
    ///
    /// Always [`MaterializationRefusal::PostFinalizationMutation`] naming
    /// the output region.
    pub const fn reblind(&self) -> Result<Self, MaterializationRefusal> {
        let _ = self;
        Err(MaterializationRefusal::PostFinalizationMutation {
            region: ProofFinalizedRegion::Outputs,
        })
    }
}

/// One complete transaction-wide confidential materialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializedConfidentialCandidate {
    proof_finalized: ProofFinalizedCandidate,
    profiles: ConfidentialMaterializationProfiles,
    opening_binding_census: OpeningBindingCensus,
    signer_inputs: Vec<ProofFinalizedSignerInput>,
}

impl MaterializedConfidentialCandidate {
    /// The frozen candidate.
    #[must_use]
    pub const fn proof_finalized(&self) -> &ProofFinalizedCandidate {
        &self.proof_finalized
    }

    /// The profiles the run was under.
    #[must_use]
    pub const fn profiles(&self) -> ConfidentialMaterializationProfiles {
        self.profiles
    }

    /// The opening-binding census.
    #[must_use]
    pub const fn opening_binding_census(&self) -> &OpeningBindingCensus {
        &self.opening_binding_census
    }

    /// Every signer input, in input order.
    #[must_use]
    pub fn signer_inputs(&self) -> &[ProofFinalizedSignerInput] {
        &self.signer_inputs
    }
}

// --- The entry point ----------------------------------------------------

/// Materializes one complete private candidate, transaction-wide.
///
/// The seven stages run in order and no stage is reordered for
/// convenience: blinders, the balancing solve, the explicit asset,
/// commitments checked against a second origin, nonces and proofs, the
/// output-witness vector, and the freeze. Stage four is not optional and
/// is not a debug assertion — it is where the two-origin rule is
/// enforced, and a build that skipped it would have one opinion and
/// report it twice.
///
/// # Errors
///
/// [`MaterializationRefusal`], at the first clause the request fails.
/// Every one is a construction refusal, and a preflight refusal names
/// what was missing rather than an amount, a subtotal, or an opening.
pub fn materialize_confidential_candidate(
    intent: &ConfidentialConstructionIntent,
    fixtures: &FrozenConfidentialFixtureView,
    crypto: &dyn ConfidentialProofMaterializer,
    checker: &dyn IndependentCommitmentCheck,
) -> Result<MaterializedConfidentialCandidate, MaterializationRefusal> {
    let opening_binding_census = preflight(intent, fixtures, checker)?;

    let asset = protocol_asset(intent)?;
    let view = destination_view(intent, fixtures)?;

    // Stages one and two: every output's value blinder, derived or solved.
    let input_blinder_sum = *fixture_of(intent, fixtures)?.input_blinder_sum();
    let derived = output_blinders(view, &input_blinder_sum, checker)?;

    // Stage three: the protocol asset is explicit, every protocol asset
    // blinder is zero, and a confidential asset result is refused
    // outright. Both are settled in the preflight; what remains here is
    // emitting the explicit field, which is the only asset field this
    // construction can build.
    let asset_field = AssetField::Explicit(asset);

    // Stages four, five, and six, per output and in order.
    let mut outputs = Vec::with_capacity(view.len());
    let mut output_witnesses = Vec::with_capacity(view.len());
    for (index, projected) in view.iter().enumerate() {
        let (output, witness) = if projected.role() == ConfidentialOutputRole::Fee {
            // The fee output takes none of stages four, five and six. It
            // has no commitment to compare against an independent
            // recomputation, no nonce to derive, and no range to prove,
            // and running any of those over it is what a blinded fee
            // output would have been.
            materialize_fee_output(index, projected, asset)?
        } else {
            let blinder = derived[index].ok_or(MaterializationRefusal::InvalidScalar {
                role: DerivationRole::ValueBlinder,
            })?;
            materialize_one_output(
                index,
                projected,
                asset,
                asset_field,
                &blinder,
                crypto,
                checker,
            )?
        };
        outputs.push(output);
        output_witnesses.push(witness);
    }

    for member in intent.non_protocol_region().members() {
        outputs.push(TargetOutput::new(
            AssetField::Explicit(member.asset()),
            ValueField::Explicit(member.amount()),
            NonceField::Null,
            member.output_program().to_vec(),
        ));
        output_witnesses.push(OutputWitness::empty());
    }

    if output_witnesses.len() != outputs.len() {
        return Err(MaterializationRefusal::OutputWitnessCensusMismatch);
    }

    let inputs: Vec<TargetInput> = intent
        .inputs()
        .iter()
        .map(|input| TargetInput::new(input.outpoint(), input.sequence()))
        .collect();
    let witnesses = vec![InputWitness::default(); inputs.len()];

    // Stage seven: freeze, before any external signing input exists.
    let protected = TargetTransaction::with_output_witnesses(
        intent.version(),
        inputs,
        outputs,
        intent.lock_time(),
        witnesses,
        output_witnesses,
    )
    .map_err(|_| MaterializationRefusal::OutputWitnessCensusMismatch)?;
    let proof_finalized = ProofFinalizedCandidate::freeze(protected)?;

    // The opening scalars of the outputs that HAVE openings. A fee output
    // contributes none, and contributing three zeroes on its behalf would
    // have put a value in this census that no opening produced.
    let mut opening_scalars: Vec<[u8; SCALAR_BYTES]> = Vec::new();
    for projected in view {
        opening_scalars.extend(projected.value_blinder().copied());
        opening_scalars.extend(projected.nonce_input().copied());
        opening_scalars.extend(projected.rangeproof_seed().copied());
    }
    let signer_inputs = signer_inputs(
        intent,
        &proof_finalized,
        &opening_binding_census,
        &opening_scalars,
    )?;

    Ok(MaterializedConfidentialCandidate {
        proof_finalized,
        profiles: intent.profiles(),
        opening_binding_census,
        signer_inputs,
    })
}

/// Stages one and two: every output's value blinder, derived or solved.
///
/// # What comes back, and what an absent entry means
///
/// One entry per output, in the view's order. `Some` for every output
/// whose blinder the materializer will commit with; `None` only for the
/// fee output, which has no blinder to commit with because its value is
/// explicit. The balancing output's entry is filled by the solve before
/// this returns, so an absent entry never means "not computed yet".
///
/// # Errors
///
/// [`MaterializationRefusal::InvalidScalar`] for an absent, zero or
/// degenerate blinder, and
/// [`MaterializationRefusal::ValueBlinderImbalance`] where the solve
/// disagrees with what the fixture registered.
fn output_blinders(
    view: &[ConfidentialFixtureOutputView],
    input_blinder_sum: &[u8; SCALAR_BYTES],
    checker: &dyn IndependentCommitmentCheck,
) -> Result<Vec<Option<[u8; SCALAR_BYTES]>>, MaterializationRefusal> {
    // Stage one: every output's derived value blinder except the
    // balancing one, which is solved rather than derived.
    let mut derived: Vec<Option<[u8; SCALAR_BYTES]>> = Vec::with_capacity(view.len());
    let mut others: Vec<[u8; SCALAR_BYTES]> = Vec::new();
    for projected in view {
        match projected.role() {
            // Neither the balancing output nor the fee output brings a
            // freely chosen blinder to `others`, and they reach that
            // through opposite facts. The balancing one's blinder is
            // SOLVED from the others and so cannot be one of them. The
            // fee one has no blinder at all: an explicit value is
            // committed with an all-zero blinder, so it contributes
            // nothing to the sum the solve subtracts.
            ConfidentialOutputRole::Balancing | ConfidentialOutputRole::Fee => derived.push(None),
            ConfidentialOutputRole::Primary => {
                let blinder =
                    *projected
                        .value_blinder()
                        .ok_or(MaterializationRefusal::InvalidScalar {
                            role: DerivationRole::ValueBlinder,
                        })?;
                if blinder == [0_u8; SCALAR_BYTES] {
                    return Err(MaterializationRefusal::InvalidScalar {
                        role: DerivationRole::ValueBlinder,
                    });
                }
                others.push(blinder);
                derived.push(Some(blinder));
            }
        }
    }

    // Stage two: solve the balancing blinder, and refuse a degenerate
    // solution rather than nudging it. The solve is the independent
    // origin's arithmetic and the comparison is against what the fixture
    // registered, so a disagreement is a blinder imbalance and not a
    // rounding difference.
    let solved = checker
        .solve_balancing_blinder(input_blinder_sum, &others)
        .ok_or(MaterializationRefusal::InvalidScalar {
            role: DerivationRole::ValueBlinder,
        })?;
    for (index, projected) in view.iter().enumerate() {
        if projected.role() == ConfidentialOutputRole::Balancing {
            if projected.value_blinder() != Some(&solved) {
                return Err(MaterializationRefusal::ValueBlinderImbalance);
            }
            derived[index] = Some(solved);
        }
    }
    Ok(derived)
}

/// One output's commitment, nonce, and proof, in the stages' own order.
///
/// Stage four is the reason this is not inlined into a loop body that
/// merely builds fields: the construction's answer and the independent
/// origin's answer are computed here, and they are compared through a
/// function that takes one of each type. A refactor that made the two
/// calls return the same type would have removed the check without
/// removing a line of it.
#[allow(clippy::too_many_arguments)]
fn materialize_one_output(
    index: usize,
    projected: &ConfidentialFixtureOutputView,
    asset: AssetId,
    asset_field: AssetField,
    blinder: &[u8; SCALAR_BYTES],
    crypto: &dyn ConfidentialProofMaterializer,
    checker: &dyn IndependentCommitmentCheck,
) -> Result<(TargetOutput, OutputWitness), MaterializationRefusal> {
    // Stage four: construct, then independently recompute, then compare
    // two different types.
    if !checker.origin().may_check_a_construction() || checker.origin() == crypto.origin() {
        return Err(
            MaterializationRefusal::IndependentCommitmentOriginNotDistinct { output: index },
        );
    }
    let materialized = crypto
        .value_commitment(asset, projected.semantic_amount(), blinder)
        .ok_or(MaterializationRefusal::InvalidCommitment { output: index })?;
    let recomputed = checker
        .recompute(asset, projected.semantic_amount(), blinder)
        .ok_or(MaterializationRefusal::InvalidCommitment { output: index })?;
    if !commitments_agree(&materialized, &recomputed) {
        return Err(MaterializationRefusal::IndependentCommitmentMismatch { output: index });
    }

    // Stage five: deterministic nonce material, then a nonempty range
    // proof bound to this value's commitment, the unblinded asset
    // generator, and the output program.
    let nonce = crypto
        .nonce_commitment(
            projected
                .nonce_input()
                .ok_or(MaterializationRefusal::NonceMaterializationFailed { output: index })?,
        )
        .ok_or(MaterializationRefusal::NonceMaterializationFailed { output: index })?;
    let request = RangeproofRequest::new(
        index,
        &materialized,
        asset,
        projected.semantic_amount(),
        blinder,
        projected
            .rangeproof_seed()
            .ok_or(MaterializationRefusal::RangeproofMaterializationFailed { output: index })?,
        projected.output_program(),
    );
    let proof = crypto
        .range_proof(&request)
        .ok_or(MaterializationRefusal::RangeproofMaterializationFailed { output: index })?;
    if !proof.binds(&request) {
        return Err(MaterializationRefusal::ProofBindingMismatch { output: index });
    }
    if proof.proof().is_empty() {
        return Err(MaterializationRefusal::RangeproofEmpty { output: index });
    }
    if !proof.surjection_proof().is_empty() {
        return Err(MaterializationRefusal::UnexpectedSurjectionProof { output: index });
    }

    let output = TargetOutput::new(
        asset_field,
        ValueField::Commitment(*materialized.bytes()),
        NonceField::Commitment(nonce),
        projected.output_program().to_vec(),
    );
    // Stage six: one output-witness entry per output, each carrying an
    // empty surjection proof and its own range proof.
    Ok((
        output,
        OutputWitness::range_proof_only(proof.proof().to_vec()),
    ))
}

/// The fee output, built as the target defines one.
///
/// # Why this is a separate stage rather than a branch inside the other
///
/// Every clause of [`materialize_one_output`] is about a commitment: an
/// independent recomputation to compare against, a nonce to derive, a
/// range to prove, a witness entry to carry the proof. A fee output has
/// none of them, and a fee output routed through that function would
/// come back BLINDED — which is not a fee at the target, and is the
/// silently wrong transaction the projection used to refuse rather than
/// build.
///
/// # What it emits, and why each part is not a choice
///
/// An explicit value, an explicit asset, a null nonce, an empty program,
/// and an empty output-witness entry. That is `CTxOut::IsFee` read
/// forwards: the target recognizes a fee by an empty scriptPubKey with
/// an explicit value and an explicit asset, so a construction that
/// wanted a fee has exactly one shape available to it. The output-witness
/// entry is empty rather than absent because the census is one entry per
/// output, and an output with nothing to prove still occupies its place.
///
/// The asset is the PROTOCOL asset, taken from the same place every
/// other output takes it. A fee in some other asset would leave the
/// protocol tally short by the fee, which the semantic conservation
/// clause would refuse — correctly, and for a reason that would read
/// like an arithmetic slip rather than like the asset choice it was.
///
/// # Errors
///
/// [`MaterializationRefusal::FeeOutputProgramNotEmpty`],
/// [`MaterializationRefusal::FeeOutputCarriesAnOpening`],
/// [`MaterializationRefusal::FeeOutputValueZero`], and
/// [`MaterializationRefusal::FeeOutputNotRecognizable`] where the built
/// output does not satisfy the target's own predicate.
fn materialize_fee_output(
    index: usize,
    projected: &ConfidentialFixtureOutputView,
    asset: AssetId,
) -> Result<(TargetOutput, OutputWitness), MaterializationRefusal> {
    if !projected.output_program().is_empty() {
        return Err(MaterializationRefusal::FeeOutputProgramNotEmpty { output: index });
    }
    if projected.value_blinder().is_some()
        || projected.nonce_input().is_some()
        || projected.rangeproof_seed().is_some()
    {
        return Err(MaterializationRefusal::FeeOutputCarriesAnOpening { output: index });
    }
    if projected.semantic_amount() == 0 {
        return Err(MaterializationRefusal::FeeOutputValueZero { output: index });
    }

    let output = TargetOutput::new(
        AssetField::Explicit(asset),
        ValueField::Explicit(projected.semantic_amount()),
        NonceField::Null,
        Vec::new(),
    );
    // Built, then asked. The predicate is the target's three conjuncts
    // and not a restatement of the clauses above: those check what
    // ARRIVED, this checks what LEFT.
    if !output.is_fee() {
        return Err(MaterializationRefusal::FeeOutputNotRecognizable { output: index });
    }
    Ok((output, OutputWitness::empty()))
}

/// Everything that happens before any cryptographic work.
///
/// None of it reports a private subtotal. A refusal here names what was
/// missing, because a refusal that leaked an amount would have published
/// exactly what the canonical request keeps off the wire.
fn preflight(
    intent: &ConfidentialConstructionIntent,
    fixtures: &FrozenConfidentialFixtureView,
    checker: &dyn IndependentCommitmentCheck,
) -> Result<OpeningBindingCensus, MaterializationRefusal> {
    // Five: the profiles, first, because an unsupported combination is
    // cheaper to refuse than anything below it and refusing it late would
    // mean doing work under a profile this build does not implement.
    if matches!(
        intent.profiles().materializer_profile,
        ConfidentialMaterializerProfile::PerOutputValueCapability
    ) {
        return Err(MaterializationRefusal::PerOutputMaterializationRefused);
    }
    if !intent.profiles().supported() {
        return Err(MaterializationRefusal::UnsupportedProfileCombination);
    }

    // Two, first half: the families are complete and nonempty.
    if intent.inputs().is_empty() || intent.destinations().is_empty() {
        return Err(MaterializationRefusal::IncompleteFamilyClassification);
    }

    // Two, second half: outpoints are unique.
    let mut seen: BTreeSet<Outpoint> = BTreeSet::new();
    for input in intent.inputs() {
        if !seen.insert(input.outpoint()) {
            return Err(MaterializationRefusal::DuplicateInputOutpoint {
                outpoint: input.outpoint(),
            });
        }
    }

    let entries = verify_predecessor_openings(intent, fixtures, checker)?;

    // Four: the destinations bind to one fixture, in its own fixed order,
    // and the bounded parity search settled rather than exhausting.
    let fixture = fixture_of(intent, fixtures)?;
    if let ParityOutcome::Exhausted { .. } = fixture.parity() {
        return Err(MaterializationRefusal::BoundedParitySearchExhausted);
    }
    if fixture.outputs().len() != intent.destinations().len() {
        return Err(MaterializationRefusal::FixtureOutputOrderMismatch);
    }
    for (index, destination) in intent.destinations().iter().enumerate() {
        if destination.fixture().output() != index {
            return Err(MaterializationRefusal::FixtureOutputOrderMismatch);
        }
        let projected = fixture
            .outputs()
            .get(index)
            .ok_or(MaterializationRefusal::FixtureOutputOrderMismatch)?;
        if projected.role() != destination.role()
            || projected.output_program() != destination.output_program()
            || projected.semantic_amount() != destination.semantic_amount()
        {
            return Err(MaterializationRefusal::FixtureOutputOrderMismatch);
        }
        if destination.explicit_asset() != fixture.explicit_asset() {
            return Err(MaterializationRefusal::ProtocolAssetMismatch {
                member: FamilyMember::ProtocolOutput(index),
            });
        }
    }

    // Three: semantic conservation closes across the protocol region,
    // with the non-protocol region excluded from both equations.
    let consumed: Option<u64> = intent.inputs().iter().try_fold(0_u64, |total, input| {
        total.checked_add(input.explicit_amount())
    });
    let created: Option<u64> = intent
        .destinations()
        .iter()
        .try_fold(0_u64, |total, destination| {
            total.checked_add(destination.semantic_amount())
        });
    match (consumed, created) {
        (Some(consumed), Some(created)) if consumed == created => {}
        _ => return Err(MaterializationRefusal::SemanticValueImbalance),
    }

    // The excluded region really is excluded: no member of it carries the
    // protocol asset, and none occupies a protocol member's place.
    for (index, member) in intent.non_protocol_region().members().iter().enumerate() {
        if member.asset() == fixture.explicit_asset() {
            return Err(MaterializationRefusal::NonProtocolRegionAffectsProtocolBalance);
        }
        if intent
            .destinations()
            .iter()
            .any(|destination| destination.output_program() == member.output_program())
        {
            return Err(MaterializationRefusal::NonProtocolRegionOverlapsProtocol {
                member: FamilyMember::NonProtocolOutput(index),
            });
        }
    }

    Ok(OpeningBindingCensus { entries })
}

/// The first preflight clause, whole.
///
/// Every handle and digest resolves uniquely against the frozen view,
/// and every predecessor opening recomputes the commitment the target was
/// observed to hold. The asset side of the protocol region is settled
/// here too, because an input whose asset is committed or wrong is not an
/// input whose opening is worth recomputing.
///
/// What comes back is the census of verified references and NOT the
/// openings it checked, which is the whole shape of the result: the
/// checking happens where the openings are, and what leaves is a
/// reference.
fn verify_predecessor_openings(
    intent: &ConfidentialConstructionIntent,
    fixtures: &FrozenConfidentialFixtureView,
    checker: &dyn IndependentCommitmentCheck,
) -> Result<Vec<VerifiedFixtureReference>, MaterializationRefusal> {
    // Every handle and digest resolves uniquely, and every
    // predecessor opening recomputes the commitment the target was
    // observed to hold.
    let mut bound: BTreeSet<(String, usize)> = BTreeSet::new();
    let mut entries = Vec::with_capacity(intent.inputs().len());
    for input in intent.inputs() {
        let reference = input.opening();
        let fixture = fixtures.fixture(reference.handle()).ok_or_else(|| {
            MaterializationRefusal::UnknownFixtureHandle {
                handle: reference.handle().to_owned(),
            }
        })?;
        if fixture.digest() != reference.digest() {
            return Err(MaterializationRefusal::FixtureDigestMismatch {
                handle: reference.handle().to_owned(),
            });
        }
        if !bound.insert((reference.handle().to_owned(), reference.output())) {
            return Err(MaterializationRefusal::FixtureBindingAmbiguous {
                handle: reference.handle().to_owned(),
            });
        }
        let projected = fixture.outputs().get(reference.output()).ok_or_else(|| {
            MaterializationRefusal::PredecessorOpeningMissing {
                outpoint: input.outpoint(),
            }
        })?;

        // The asset side of the protocol region: explicit, the protocol
        // asset, and a zero blinder.
        let member = FamilyMember::Input(entries.len());
        match input.observed_asset() {
            AssetField::Commitment(_) => {
                return Err(MaterializationRefusal::ConfidentialProtocolAsset { member });
            }
            AssetField::Explicit(observed) => {
                if observed != fixture.explicit_asset() {
                    return Err(MaterializationRefusal::ProtocolAssetMismatch { member });
                }
            }
        }
        if *input.zero_asset_blinder() != [0_u8; SCALAR_BYTES] {
            return Err(MaterializationRefusal::NonzeroProtocolAssetBlinder { member });
        }

        let observed = match input.observed_value() {
            ValueField::Commitment(commitment) => commitment,
            ValueField::Explicit(_) => {
                return Err(MaterializationRefusal::PredecessorOpeningMismatch {
                    outpoint: input.outpoint(),
                });
            }
        };
        // A consumed predecessor output must have an opening. A fee
        // output has none and is unspendable besides, so an input that
        // resolved to one is a reference that does not name a coin.
        let consumed_blinder = projected.value_blinder().ok_or_else(|| {
            MaterializationRefusal::PredecessorOpeningMismatch {
                outpoint: input.outpoint(),
            }
        })?;
        let recomputed = checker
            .recompute(
                fixture.explicit_asset(),
                projected.semantic_amount(),
                consumed_blinder,
            )
            .ok_or_else(|| MaterializationRefusal::PredecessorOpeningMismatch {
                outpoint: input.outpoint(),
            })?;
        if *recomputed.bytes() != observed {
            return Err(MaterializationRefusal::PredecessorOpeningMismatch {
                outpoint: input.outpoint(),
            });
        }
        if projected.semantic_amount() != input.explicit_amount() {
            return Err(MaterializationRefusal::PredecessorOpeningMismatch {
                outpoint: input.outpoint(),
            });
        }

        entries.push(VerifiedFixtureReference {
            outpoint: input.outpoint(),
            handle: reference.handle().to_owned(),
            digest: *reference.digest(),
            output: reference.output(),
        });
    }

    Ok(entries)
}

/// The one fixture every destination binds to.
fn fixture_of<'view>(
    intent: &ConfidentialConstructionIntent,
    fixtures: &'view FrozenConfidentialFixtureView,
) -> Result<&'view ConfidentialFixtureView, MaterializationRefusal> {
    let first = intent
        .destinations()
        .first()
        .ok_or(MaterializationRefusal::IncompleteFamilyClassification)?;
    let handle = first.fixture().handle();
    for destination in intent.destinations() {
        if destination.fixture().handle() != handle {
            return Err(MaterializationRefusal::FixtureOutputOrderMismatch);
        }
    }
    let fixture =
        fixtures
            .fixture(handle)
            .ok_or_else(|| MaterializationRefusal::UnknownFixtureHandle {
                handle: handle.to_owned(),
            })?;
    if fixture.digest() != first.fixture().digest() {
        return Err(MaterializationRefusal::FixtureDigestMismatch {
            handle: handle.to_owned(),
        });
    }
    Ok(fixture)
}

/// The projected outputs the destinations realize, in fixed order.
fn destination_view<'view>(
    intent: &ConfidentialConstructionIntent,
    fixtures: &'view FrozenConfidentialFixtureView,
) -> Result<&'view [ConfidentialFixtureOutputView], MaterializationRefusal> {
    Ok(fixture_of(intent, fixtures)?.outputs())
}

/// The one explicit protocol asset the whole protocol region carries.
fn protocol_asset(
    intent: &ConfidentialConstructionIntent,
) -> Result<AssetId, MaterializationRefusal> {
    let first = intent
        .destinations()
        .first()
        .ok_or(MaterializationRefusal::IncompleteFamilyClassification)?;
    let asset = first.explicit_asset();
    for (index, destination) in intent.destinations().iter().enumerate() {
        if destination.explicit_asset() != asset {
            return Err(MaterializationRefusal::ProtocolAssetMismatch {
                member: FamilyMember::ProtocolOutput(index),
            });
        }
    }
    Ok(asset)
}

/// The signer inputs, checked for anything that would expose an opening.
///
/// The check is not a lint. A caller that passed an opening in as
/// observed target data would produce a signing input carrying it, and
/// the way to keep that from being reported is to refuse to construct the
/// value at all.
fn signer_inputs(
    intent: &ConfidentialConstructionIntent,
    candidate: &ProofFinalizedCandidate,
    census: &OpeningBindingCensus,
    openings: &[[u8; SCALAR_BYTES]],
) -> Result<Vec<ProofFinalizedSignerInput>, MaterializationRefusal> {
    if census.entries().len() != intent.inputs().len() {
        return Err(MaterializationRefusal::OpeningBindingCensusMismatch);
    }
    let mut inputs = Vec::with_capacity(intent.inputs().len());
    for (index, input) in intent.inputs().iter().enumerate() {
        if openings
            .iter()
            .any(|opening| carries_scalar(input.observed_program(), opening))
        {
            return Err(MaterializationRefusal::SignerInputWouldExposeOpening {
                outpoint: input.outpoint(),
            });
        }
        inputs.push(ProofFinalizedSignerInput {
            outpoint: input.outpoint(),
            spent_asset: input.observed_asset(),
            spent_value: input.observed_value(),
            spent_program: input.observed_program().to_vec(),
            position: u16::try_from(index).unwrap_or(u16::MAX),
            byte_binding: candidate.protected_bytes().to_vec(),
        });
    }
    Ok(inputs)
}

/// Whether a byte string contains one thirty-two byte scalar.
fn carries_scalar(bytes: &[u8], scalar: &[u8; SCALAR_BYTES]) -> bool {
    bytes.len() >= SCALAR_BYTES && bytes.windows(SCALAR_BYTES).any(|window| window == scalar)
}
