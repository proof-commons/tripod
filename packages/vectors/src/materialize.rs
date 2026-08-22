//! Target materialization, which Guide-12 §17.2 keeps separate.
//!
//! A semantic fixture states amounts and a sponsor case. Everything
//! concrete — outpoints, the public input view, the shape, the witness,
//! the bytes — is produced here, from the bundle, the ABI, the typed
//! request, and the public target input view, exactly as §17.2's
//! four-term recipe says.
//!
//! # Where the outpoints come from
//!
//! §17.2 excludes target indices from the fixture, so the fixture cannot
//! carry outpoints. They arrive separately, in an [`AshFunding`] record
//! the caller supplies — from a ceremony's own answers for a vector that
//! will be submitted, and from a named placeholder for one that will
//! not. Materialization stays a pure function of its inputs either way,
//! so repeated materialization is byte-identical, which is the property
//! §18.1's last positive class is about.
//!
//! # What this is not
//!
//! Nothing here executes anything. `construct` settles a candidate
//! transaction and this module records its bytes and resources; no
//! target has seen them. §1.5 files a refusal from here as an
//! ABI/construction rejection, never as a target verdict.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU8;

use linker::backend::{CompactAshShape, CompactAshShapeBounds, SponsorChangePresence};
use transaction::{
    AssetField, AssetId, CompactAshRequest, Outpoint, PublicConstructionView, PublicOutputView,
    SponsorCapability, SponsorOffer, SponsorSignature, SponsorSigningRequest, SyntheticDisclaimer,
    TransactionRefusal, Txid, ValueField, construct,
};

use crate::bundle::FixtureBundle;
use crate::error::VectorError;
use crate::fixture::{CompactAshSemanticCase, SemanticFixtureId};
use crate::matrix::{EvidenceBoundary, VectorClass};
use crate::projection::SponsorChange;

/// The demonstration shape bounds the fixture bundle was linked under.
///
/// Restated here rather than reached for, because `CompactAshShape::new`
/// needs the bounds as a value and the linked bundle publishes shapes
/// rather than the bounds that generated them. The census test below
/// checks the restatement against the ABI's own shape set, so a drift
/// fails rather than silently narrowing the matrix.
const MAXIMUM_ASH_INPUTS: u8 = 4;
/// The maximum number of sponsor members the demonstration bounds admit.
const MAXIMUM_SPONSORS: u8 = 1;

/// A target vector's identity: one semantic fixture at one shape.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TargetVectorId {
    fixture: SemanticFixtureId,
    ash_inputs: u8,
    sponsors: u8,
}

impl TargetVectorId {
    /// The fixture this vector materializes.
    #[must_use]
    pub const fn fixture(&self) -> SemanticFixtureId {
        self.fixture
    }

    /// How many ASH inputs the shape carries.
    #[must_use]
    pub const fn ash_inputs(&self) -> u8 {
        self.ash_inputs
    }

    /// How many sponsor members the shape carries.
    #[must_use]
    pub const fn sponsors(&self) -> u8 {
        self.sponsors
    }
}

/// A materialized target vector: exact bytes and the resources they
/// settle, bound to the fixture that produced them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaterializedTargetVector {
    id: TargetVectorId,
    class: VectorClass,
    expected: EvidenceBoundary,
    shape: CompactAshShape,
    bytes: Vec<u8>,
    weight: u64,
    virtual_size: u64,
    witness_bytes: u64,
    successor_amount: u64,
    disclaimers: Vec<SyntheticDisclaimer>,
}

impl MaterializedTargetVector {
    /// The vector's identity.
    #[must_use]
    pub const fn id(&self) -> TargetVectorId {
        self.id
    }

    /// The §18 class this vector answers.
    #[must_use]
    pub const fn class(&self) -> VectorClass {
        self.class
    }

    /// The boundary a run of this vector is expected to reach.
    #[must_use]
    pub const fn expected(&self) -> EvidenceBoundary {
        self.expected
    }

    /// The shape the ABI classified this transaction as.
    #[must_use]
    pub const fn shape(&self) -> CompactAshShape {
        self.shape
    }

    /// The exact target transaction bytes.
    ///
    /// These are the bytes Wave 11 submits. They are not evidence of
    /// anything yet: no target has seen them.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The transaction's weight.
    #[must_use]
    pub const fn weight(&self) -> u64 {
        self.weight
    }

    /// The transaction's virtual size.
    #[must_use]
    pub const fn virtual_size(&self) -> u64 {
        self.virtual_size
    }

    /// The witness byte count.
    #[must_use]
    pub const fn witness_bytes(&self) -> u64 {
        self.witness_bytes
    }

    /// The successor amount the constructor settled on.
    #[must_use]
    pub const fn settled_successor(&self) -> u64 {
        self.successor_amount
    }

    /// The synthetic disclaimers the construction carries.
    ///
    /// Non-empty for every vector in this package: every ASH input is
    /// synthetic test funding, and §15.9 requires that to travel with
    /// the artifact rather than with the prose about it.
    #[must_use]
    pub fn disclaimers(&self) -> &[SyntheticDisclaimer] {
        &self.disclaimers
    }
}

/// Where the ASH inputs of one vector come from.
///
/// # Why this replaced a derived rule
///
/// Wave 10 derived outpoints from the fixture's ordinal and the
/// member's position, which made materialization a pure function of the
/// semantic case and made repeated materialization byte-identical. It
/// also made the transactions unspendable: a derived outpoint names a
/// coin no chain ever created, so every one of them would be refused
/// for a missing input before any script ran.
///
/// Wave 11 takes them from the ceremony instead. The coins exist
/// because a target created them, and a vector spends the ones it was
/// given. Determinism is not lost, only relocated: materialization is
/// still a pure function of its inputs, and the same funding answers
/// produce the same bytes twice.
///
/// # Why this is a distinct type and not a bare slice
///
/// A slice of outpoints could be handed to the wrong vector. This
/// carries the vector's own identity alongside them, so materialization
/// can refuse a funding record that was not cut for it rather than
/// silently building a transaction against another vector's coins.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AshFunding {
    vector: TargetVectorId,
    outpoints: Vec<Outpoint>,
}

impl AshFunding {
    /// States the coins one vector's ASH inputs spend.
    ///
    /// # Errors
    ///
    /// [`VectorError::FundingCardinalityMismatch`] when the ceremony
    /// supplied a number of coins the vector's shape does not take, and
    /// [`VectorError::DuplicateFundedOutpoint`] when it supplied one
    /// coin twice — which would turn a positive vector into §18.3's
    /// first negative one without anything saying so.
    pub fn new(vector: TargetVectorId, outpoints: Vec<Outpoint>) -> Result<Self, VectorError> {
        let wanted = usize::from(vector.ash_inputs());
        if outpoints.len() != wanted {
            return Err(VectorError::FundingCardinalityMismatch {
                vector,
                wanted,
                supplied: outpoints.len(),
            });
        }
        let distinct: std::collections::BTreeSet<Outpoint> = outpoints.iter().copied().collect();
        if distinct.len() != outpoints.len() {
            return Err(VectorError::DuplicateFundedOutpoint(vector));
        }
        Ok(Self { vector, outpoints })
    }

    /// The vector these coins were cut for.
    #[must_use]
    pub const fn vector(&self) -> TargetVectorId {
        self.vector
    }

    /// The coins, in the order the ceremony reported them.
    #[must_use]
    pub fn outpoints(&self) -> &[Outpoint] {
        &self.outpoints
    }

    /// Coins no chain created, for a vector nothing will submit.
    ///
    /// # Why an unexecuted fixture still needs outpoints
    ///
    /// The canonical fixtures exist to carry exact bytes: the reference
    /// oracle decodes them, re-encodes them, hashes their leaves, and
    /// checks their control blocks against the pinned key. None of that
    /// needs the inputs to be real, and all of it needs them to be
    /// *stated*, because a transaction with no inputs is not the
    /// transaction whose encoding is under test.
    ///
    /// # Why it is named rather than silent
    ///
    /// Wave 10 derived these inside materialization, where nothing
    /// distinguished them from coins a ceremony had supplied. They are
    /// not the same thing and a report must not be able to confuse
    /// them: an output at one of these outpoints does not exist, so a
    /// vector built on it can only ever be refused for a missing input,
    /// before any script runs. Requiring a caller to ask for it by this
    /// name is what keeps that fact attached to the artifact.
    #[must_use]
    pub fn unexecutable_placeholder(vector: TargetVectorId) -> Self {
        let ordinal = vector.fixture().ordinal();
        let mut outpoints = Vec::with_capacity(usize::from(vector.ash_inputs()));
        for member in 0..u32::from(vector.ash_inputs()) {
            let mut seed = [0_u8; 32];
            seed[0] = 0xa0;
            seed[1..5].copy_from_slice(&ordinal.to_be_bytes());
            // The width is fixed by the shape bounds, so both the index
            // and the outpoint are constructible; a refusal here would
            // mean the bounds and this rule had drifted apart, and the
            // empty vector it would leave is refused by `new` below.
            if let Ok(outpoint) = Outpoint::new(Txid::from_internal(seed), member) {
                outpoints.push(outpoint);
            }
        }
        Self { vector, outpoints }
    }
}

/// The identity the sponsorless vector of one semantic case carries.
///
/// Stated once here because three callers need it before a vector
/// exists: the plan, to name the work it is waiting for; the ceremony,
/// to cut funding for it; and materialization itself. A second
/// derivation would be a second authored spelling of one identity
/// `(´[PLAN-rule:guide12-exec:typed-source]´)`.
/// # Why the sponsor count comes from the case
///
/// It used to be zero here, because no sponsored row could be built at
/// all and every identity this function produced was a sponsorless
/// one. A sponsored row's identity has to carry its own member count or
/// two rows differing only in the size of their sponsor region would
/// share an identity — and the funding cut for one would be accepted
/// for the other.
#[must_use]
pub fn vector_id(case: &CompactAshSemanticCase) -> TargetVectorId {
    TargetVectorId {
        fixture: case.id(),
        ash_inputs: u8::try_from(case.ash_inputs()).unwrap_or(u8::MAX),
        sponsors: u8::try_from(case.sponsor().members()).unwrap_or(u8::MAX),
    }
}

fn shape_of(
    ash_inputs: u8,
    sponsors: u8,
    change: bool,
    id: TargetVectorId,
) -> Result<CompactAshShape, VectorError> {
    let bounds = CompactAshShapeBounds::new(
        NonZeroU8::new(MAXIMUM_ASH_INPUTS).unwrap_or(NonZeroU8::MIN),
        MAXIMUM_SPONSORS,
    )
    .map_err(|_| VectorError::MaterializedShapeMismatch(id))?;
    let ash = NonZeroU8::new(ash_inputs).ok_or(VectorError::MaterializedShapeMismatch(id))?;
    CompactAshShape::new(
        bounds,
        ash,
        sponsors,
        if change {
            SponsorChangePresence::Present
        } else {
            SponsorChangePresence::Absent
        },
    )
    .map_err(|_| VectorError::MaterializedShapeMismatch(id))
}

/// What one sponsor coin is, as the executor reported it.
///
/// # Every field is the executor's answer
///
/// The program especially. A sponsor coin has to sit at a program its
/// holder can authorize a spend of, so the caller cannot choose it; it
/// arrives here as whatever the executor said it created, and the
/// construction is checked against the reviewed sponsor profile rather
/// than trusted to be of the admitted class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SponsorCoin {
    outpoint: Outpoint,
    amount: u64,
    program: Vec<u8>,
}

impl SponsorCoin {
    /// States one coin an executor reported creating.
    #[must_use]
    pub const fn new(outpoint: Outpoint, amount: u64, program: Vec<u8>) -> Self {
        Self {
            outpoint,
            amount,
            program,
        }
    }

    /// Where the coin is.
    #[must_use]
    pub const fn outpoint(&self) -> Outpoint {
        self.outpoint
    }

    /// What the executor said it holds.
    #[must_use]
    pub const fn amount(&self) -> u64 {
        self.amount
    }

    /// The program it sits at, as the executor reported it.
    #[must_use]
    pub fn program(&self) -> &[u8] {
        &self.program
    }
}

/// The sponsor side of one construction, answered out of process.
///
/// # Why the authorization is replayed rather than fetched
///
/// [`SponsorCapability::sign`] is a synchronous call and the party that
/// can answer it is a separate process reached one step at a time. So
/// the construction is run twice: once to learn what the authorization
/// would be about, and once with the answer in hand. Both runs are the
/// same pure function of the same inputs, and the builder's own byte
/// comparison is what enforces that — a second run that produced
/// different bytes would produce an authorization bound to the first
/// run's, and `SponsorSignatureBindingMismatch` refuses it.
///
/// The first run is not a construction anybody may use. It exists
/// inside [`sponsor_signing_requests`], which returns requests and
/// never a transaction, so the placeholder-authorized bytes it builds
/// have no path out of that function.
struct CeremonySponsor<'a> {
    coins: &'a [SponsorCoin],
    /// What the coins amount to as an offer.
    ///
    /// Built once, where a refusal has somewhere to go: the capability
    /// answers `offer` infallibly, so a duplicated coin has to be
    /// refused before a sponsor exists rather than inside the answer.
    offer: SponsorOffer,
    /// How many inputs precede the sponsor suffix.
    ///
    /// The builder numbers a signing request by its position in the
    /// whole input list, and the sponsor region is the suffix, so this
    /// is what turns a request's index back into which sponsor coin it
    /// is about.
    ash_inputs: usize,
    /// What to answer a signing request with.
    answers: SponsorAnswers<'a>,
}

/// How a [`CeremonySponsor`] answers the requests it is handed.
enum SponsorAnswers<'a> {
    /// Record what was asked, and answer with a stack of the admitted
    /// width that authorizes nothing.
    ///
    /// The placeholder is what lets *every* request be collected rather
    /// than only the first: a decline stops the builder at the input it
    /// was declined for, and a run with two sponsor members would learn
    /// about one of them. It never leaves the first run.
    Recording(std::cell::RefCell<Vec<SponsorSigningRequest>>),
    /// Answer with the authorizations the executor produced, found by
    /// the coin each one is about rather than by arrival order.
    Replaying(&'a BTreeMap<Outpoint, SponsorSignature>),
}

impl SponsorCapability for CeremonySponsor<'_> {
    fn offer(&self) -> SponsorOffer {
        self.offer.clone()
    }

    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        None
    }

    fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        match &self.answers {
            SponsorAnswers::Recording(recorded) => {
                recorded.borrow_mut().push(request.clone());
                // A stack of the admitted width carrying nothing. It
                // authorizes no spend of anything, and the run it
                // belongs to returns requests rather than a
                // transaction, so it cannot reach a target.
                Some(SponsorSignature::new(
                    request.transaction().to_vec(),
                    vec![Vec::new(), Vec::new()],
                ))
            }
            // Which coin the request is about, recovered from the index
            // rather than from arrival order, so a caller that supplied
            // its authorizations in some other order still gets each one
            // matched to the input it was produced for.
            SponsorAnswers::Replaying(signatures) => self
                .sponsor_position(request)
                .and_then(|coin| signatures.get(&coin).cloned()),
        }
    }
}

impl CeremonySponsor<'_> {
    /// The coin one signing request is about.
    ///
    /// The offer's inputs reach the builder as a sorted set, so the
    /// suffix is in outpoint order and the request's index names a
    /// position in it. Recomputing that order here rather than assuming
    /// the caller's is what keeps the two sides from disagreeing about
    /// which coin input *n* is.
    fn sponsor_position(&self, request: &SponsorSigningRequest) -> Option<Outpoint> {
        let ordered: BTreeSet<Outpoint> = self.coins.iter().map(SponsorCoin::outpoint).collect();
        let offset = usize::from(request.input()).checked_sub(self.ash_inputs)?;
        ordered.into_iter().nth(offset)
    }
}

/// Materialize one semantic case into exact target bytes.
///
/// # Errors
///
/// [`VectorError::MaterializedShapeMismatch`] when the case's own facts
/// do not name a shape the demonstration bounds admit,
/// [`VectorError::TargetMaterializationFailed`] when the constructor
/// refuses — which is an ABI/construction rejection and never a target
/// verdict — [`VectorError::SuccessorAmountMismatch`] when the amount
/// the constructor settled on is not the one the realization layer's
/// arithmetic derived, and [`VectorError::FundingNamesAnotherVector`]
/// when the funding offered was cut for a different vector.
pub fn materialize(
    fixture: &FixtureBundle,
    case: &CompactAshSemanticCase,
    funding: &AshFunding,
) -> Result<MaterializedTargetVector, VectorError> {
    build_vector(fixture, case, funding, None)
}

/// One authorization a sponsored construction needs.
///
/// The request and the coin it is about, paired here because the
/// correspondence is computed from the builder's own input numbering
/// and the offer's sorted order. A caller working it out again would be
/// a second authored spelling of one rule, and the two could disagree
/// about which coin input *n* spends.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SponsorSigningTask {
    coin: SponsorCoin,
    request: SponsorSigningRequest,
}

impl SponsorSigningTask {
    /// The coin whose spend is being authorized.
    #[must_use]
    pub const fn coin(&self) -> &SponsorCoin {
        &self.coin
    }

    /// What the authorization must be produced against.
    #[must_use]
    pub const fn request(&self) -> &SponsorSigningRequest {
        &self.request
    }
}

/// What a sponsored row's authorizations would be about.
///
/// Runs the construction once with a placeholder sponsor so that every
/// [`SponsorSigningRequest`] the builder would issue is collected, and
/// returns those requests. The transaction that run produced is dropped
/// here and has no way out: a placeholder-authorized transaction is not
/// a vector and must never be able to become one.
///
/// The requests carry the exact finalized bytes an authorization has to
/// be produced against, which is what the executor is handed and what
/// its answer is later compared with.
///
/// # Errors
///
/// Everything [`materialize`] refuses, for the same reasons.
pub fn sponsor_signing_requests(
    fixture: &FixtureBundle,
    case: &CompactAshSemanticCase,
    funding: &AshFunding,
    coins: &[SponsorCoin],
) -> Result<Vec<SponsorSigningTask>, VectorError> {
    let recorded = std::cell::RefCell::new(Vec::new());
    let sponsor = CeremonySponsor {
        coins,
        offer: ceremony_offer(coins, case.sponsor().change()).map_err(|cause| {
            VectorError::TargetMaterializationFailed {
                vector: vector_id(case),
                cause,
            }
        })?,
        ash_inputs: case.ash_inputs(),
        answers: SponsorAnswers::Recording(recorded),
    };
    build_vector(fixture, case, funding, Some(&sponsor))?;
    let SponsorAnswers::Recording(recorded) = &sponsor.answers else {
        // Unreachable: the value was just built with this arm. Answered
        // rather than unwrapped, so no path here can panic.
        return Err(VectorError::MaterializedShapeMismatch(vector_id(case)));
    };

    let mut tasks = Vec::new();
    for request in recorded.borrow().iter() {
        let outpoint = sponsor
            .sponsor_position(request)
            .ok_or_else(|| VectorError::MaterializedShapeMismatch(vector_id(case)))?;
        let coin = coins
            .iter()
            .find(|coin| coin.outpoint() == outpoint)
            .ok_or_else(|| VectorError::MaterializedShapeMismatch(vector_id(case)))?;
        tasks.push(SponsorSigningTask {
            coin: coin.clone(),
            request: request.clone(),
        });
    }
    Ok(tasks)
}

/// Materialize one sponsored case, with the authorizations in hand.
///
/// # Errors
///
/// Everything [`materialize`] refuses, plus
/// [`VectorError::TargetMaterializationFailed`] carrying
/// `SponsorSignatureMissing` when no authorization was supplied for a
/// coin the construction asks about, and
/// `SponsorSignatureBindingMismatch` when the authorization supplied was
/// produced against other bytes than this construction settled on —
/// which is the check that makes running the construction twice sound
/// rather than assumed.
pub fn materialize_sponsored(
    fixture: &FixtureBundle,
    case: &CompactAshSemanticCase,
    funding: &AshFunding,
    coins: &[SponsorCoin],
    signatures: &BTreeMap<Outpoint, SponsorSignature>,
) -> Result<MaterializedTargetVector, VectorError> {
    let sponsor = CeremonySponsor {
        coins,
        offer: ceremony_offer(coins, case.sponsor().change()).map_err(|cause| {
            VectorError::TargetMaterializationFailed {
                vector: vector_id(case),
                cause,
            }
        })?,
        ash_inputs: case.ash_inputs(),
        answers: SponsorAnswers::Replaying(signatures),
    };
    build_vector(fixture, case, funding, Some(&sponsor))
}

/// What the sponsor region's coins hold, as the executor reported them.
///
/// Summed from the amounts the executor reported rather than from what
/// was asked for, because those are the amounts the chain actually
/// holds.
fn sponsor_total(coins: &[SponsorCoin]) -> u64 {
    coins
        .iter()
        .map(SponsorCoin::amount)
        .try_fold(0_u64, u64::checked_add)
        .unwrap_or(u64::MAX)
}

/// How a row's sponsor contribution divides into fee and residual.
///
/// # Why the split is settled here and not stated by a fixture
///
/// The reserve asset has to balance across the transaction, and the fee
/// and the change output are the only places it goes — so the two
/// figures are one decision, made against amounts the executor reported.
/// A fixture states whether a residual *exists*; §17.4 rules individual
/// sponsor amounts absent from the semantic layer, so how large it is
/// cannot come from there and does not.
///
/// A change-present row halves the contribution, with the odd unit
/// going to the fee. Half of a figure already chosen to sit comfortably
/// above any relay threshold is still comfortably above it, and an
/// even split is the one choice that needs no second constant to
/// justify.
///
/// # Errors
///
/// [`TransactionRefusal::SponsorChangeWithoutSponsor`] where a residual
/// was asked for and the contribution cannot carry a nonzero one. The
/// builder omits a zero-valued change output — the target refuses a
/// spendable zero — so such a row would silently become a change-absent
/// transaction wearing a change-present class's name.
fn sponsor_split(
    coins: &[SponsorCoin],
    change: SponsorChange,
) -> Result<(u64, Option<u64>), TransactionRefusal> {
    let total = sponsor_total(coins);
    match change {
        SponsorChange::Absent => Ok((total, None)),
        SponsorChange::Present => {
            let residual = total / 2;
            if residual == 0 {
                return Err(TransactionRefusal::SponsorChangeWithoutSponsor);
            }
            Ok((total - residual, Some(residual)))
        }
    }
}

/// What the ceremony's coins offer a construction.
///
/// # Errors
///
/// [`TransactionRefusal::DuplicateSponsorOutpoint`] when the coins
/// supplied name one outpoint twice, which is a sponsor region the
/// executor did not cut, and everything [`sponsor_split`] refuses.
///
/// The change role is the row's own, not this function's choice: a
/// fixture filed under `sponsor-change-present` states that its world
/// has a residual, and an offer built without one would produce a
/// transaction of the other class entirely (`G13-R10`).
fn ceremony_offer(
    coins: &[SponsorCoin],
    change: SponsorChange,
) -> Result<SponsorOffer, TransactionRefusal> {
    let (fee, residual) = sponsor_split(coins, change)?;
    SponsorOffer::new(
        coins.iter().map(SponsorCoin::outpoint),
        fee,
        residual.map(ValueField::Explicit),
    )
}

/// The construction both entry points run.
fn build_vector(
    fixture: &FixtureBundle,
    case: &CompactAshSemanticCase,
    funding: &AshFunding,
    sponsor: Option<&CeremonySponsor<'_>>,
) -> Result<MaterializedTargetVector, VectorError> {
    let ash_inputs = u8::try_from(case.ash_inputs()).unwrap_or(u8::MAX);
    let id = vector_id(case);
    // The funding was cut for a vector, and this is that vector or it is
    // not. Building against another vector's coins would produce a
    // transaction nobody planned.
    if funding.vector() != id {
        return Err(VectorError::FundingNamesAnotherVector {
            wanted: id,
            supplied: funding.vector(),
        });
    }
    let sponsors = sponsor.map_or(0, |sponsor| {
        u8::try_from(sponsor.coins.len()).unwrap_or(u8::MAX)
    });
    // The row states how many sponsor members its world has, and the
    // coins offered have to be that many. A construction over some other
    // number would settle a shape the fixture did not name.
    if u16::from(sponsors) != case.sponsor().members() {
        return Err(VectorError::MaterializedShapeMismatch(id));
    }
    // The change role is the fixture's own fact, not a constant: the
    // shape a row selects is a conjunct of its counts and its change
    // presence, and hardcoding one of the two made both §18.1
    // sponsor-change classes select the same shape (`G13-R10`).
    let shape = shape_of(
        ash_inputs,
        sponsors,
        case.sponsor().change().is_present(),
        id,
    )?;

    let mut views = Vec::with_capacity(case.ash_inputs() + usize::from(sponsors));
    let program = fixture
        .pin()
        .output_script(fixture.target())
        .map_err(|cause| VectorError::TargetMaterializationFailed { vector: id, cause })?;

    // The asset is the bundle's own, not this module's constant: a
    // ceremony-bound bundle is linked against the asset the ceremony
    // issued, and an input declaring any other one would be refused by
    // the very leaf that introspects it.
    let asset = AssetField::Explicit(AssetId::from_internal(fixture.closed_asset()));
    let outpoints: Vec<Outpoint> = funding.outpoints().to_vec();
    for (outpoint, amount) in outpoints.iter().zip(case.inputs().iter()) {
        views.push(PublicOutputView::new(
            *outpoint,
            asset,
            ValueField::Explicit(amount.get()),
            program.clone(),
        ));
    }

    // The sponsor coins enter the public view as the executor described
    // them: its asset, its amounts, its programs. The reserve asset is
    // the bundle's own for the same reason the closed asset is — the
    // coordinator introspects it against the value linked into the leaf.
    let reserve = AssetField::Explicit(AssetId::from_internal(fixture.reserve_asset()));
    if let Some(sponsor) = sponsor {
        for coin in sponsor.coins {
            views.push(PublicOutputView::new(
                coin.outpoint(),
                reserve,
                ValueField::Explicit(coin.amount()),
                coin.program().to_vec(),
            ));
        }
    }

    let request = CompactAshRequest::new(outpoints, sponsor.is_some())
        .map_err(|cause| VectorError::TargetMaterializationFailed { vector: id, cause })?;
    let view = PublicConstructionView::new(views)
        .map_err(|cause| VectorError::TargetMaterializationFailed { vector: id, cause })?;

    let capability = sponsor.map(|sponsor| sponsor as &dyn SponsorCapability);
    let built = construct(fixture.target(), fixture.abi(), &request, &view, capability)
        .map_err(|cause| VectorError::TargetMaterializationFailed { vector: id, cause })?;

    let report = built.report();
    if report.shape() != shape {
        return Err(VectorError::MaterializedShapeMismatch(id));
    }

    // §17.3's comparison, made here rather than assumed: the expectation
    // came from the realization layer's checked sum, the settled amount
    // came from the constructor, and they are two computations.
    let expected = case.expected().successor().1.get();
    if report.successor_amount() != expected {
        return Err(VectorError::SuccessorAmountMismatch {
            vector: id,
            expected,
            settled: report.successor_amount(),
        });
    }

    let resources = report.resources();
    Ok(MaterializedTargetVector {
        id,
        class: case.class(),
        expected: case.class().boundary(),
        shape,
        bytes: built.bytes(),
        weight: resources.weight(),
        virtual_size: resources.virtual_size(),
        witness_bytes: resources.witness_bytes(),
        successor_amount: report.successor_amount(),
        disclaimers: report.disclaimers().iter().copied().collect(),
    })
}

/// Whether this candidate emitted a program for the shape a row names.
///
/// Derived from [`CompactAshShape::new`] against the demonstration
/// bounds rather than written down. The bounds admit sponsor regions up
/// to one member, and §18's matrix names a row with two, so that row
/// has no program in this bundle and this says so by computing it. A
/// candidate linked under wider bounds reclassifies the same row with
/// nothing here to edit.
///
/// A row refused here is *unbuilt*, which is what
/// `ShapeRejection::SponsorInputsAboveBound` means: not semantically
/// invalid, and not a target divergence either — the target was never
/// asked. It is a limit of this candidate's emitted shape set, and
/// keeping it distinct from the target's own money bound is the whole
/// reason the two are computed in different places.
#[must_use]
pub fn has_candidate_program(case: &CompactAshSemanticCase) -> bool {
    shape_of(
        u8::try_from(case.ash_inputs()).unwrap_or(u8::MAX),
        u8::try_from(case.sponsor().members()).unwrap_or(u8::MAX),
        case.sponsor().change().is_present(),
        vector_id(case),
    )
    .is_ok()
}

/// Whether building this row needs an authorization from outside.
///
/// A sponsored row carries an input somebody has to authorize, and no
/// part of this workspace can. The authorization arrives from the
/// executor during a run, which is why such a row can be *executed*
/// while not being a fixture whose bytes this package can state.
#[must_use]
pub const fn needs_authorization(case: &CompactAshSemanticCase) -> bool {
    case.sponsor().is_present()
}

/// Whether a row's exact bytes are a function of the fixture and its
/// funding alone.
///
/// # Why this is narrower than having a program
///
/// The canonical plan materializes every row it admits to exact bytes,
/// and those bytes are what the reference oracle decodes, re-encodes,
/// and hashes. That only means anything while the bytes are determined
/// by things this package holds. A sponsored row's witness carries an
/// authorization produced elsewhere, so its bytes are not a function of
/// the fixture at all — two runs against two executors would produce
/// two different transactions from one row, both correct.
///
/// So a sponsored row is executable and is not a byte fixture, and the
/// two questions are asked separately. Answering them with one
/// predicate is what would let a canonical fixture carry bytes nobody
/// could reproduce, or an authorized row be dropped from a run that can
/// perfectly well perform it.
#[must_use]
pub fn is_materializable(case: &CompactAshSemanticCase) -> bool {
    has_candidate_program(case) && !needs_authorization(case)
}

#[cfg(test)]
mod tests {
    use super::{AshFunding, is_materializable, materialize, vector_id};
    use crate::bundle::fixture_bundle;
    use crate::fixture::{CompactAshSemanticCase, positive_semantic_census};
    use crate::matrix::EvidenceBoundary;
    use std::collections::{BTreeMap, BTreeSet};
    use transaction::{Outpoint, SyntheticDisclaimer, TargetTransaction, Txid, check_weight};

    /// The placeholder funding the canonical fixtures materialize under.
    fn placeholder(case: &CompactAshSemanticCase) -> AshFunding {
        AshFunding::unexecutable_placeholder(vector_id(case))
    }

    #[test]
    fn every_sponsorless_positive_case_materializes() {
        let fixture = fixture_bundle().expect("the fixture bundle builds");
        let census = positive_semantic_census().expect("the positive census builds");
        let materializable: Vec<_> = census
            .iter()
            .filter(|case| is_materializable(case))
            .collect();
        assert_eq!(
            materializable.len(),
            9,
            "nine of the fourteen positive classes are byte fixtures"
        );

        for case in materializable {
            let vector = materialize(&fixture, case, &placeholder(case))
                .unwrap_or_else(|error| panic!("{:?} did not materialize: {error:?}", case.id()));
            assert_ne!(vector.bytes(), [] as [u8; 0]);
            assert_eq!(vector.expected(), EvidenceBoundary::AcceptedTransaction);
            assert_eq!(
                vector.settled_successor(),
                case.expected().successor().1.get()
            );
        }
    }

    #[test]
    fn the_materialized_bytes_decode_back_to_the_same_transaction() {
        // §1.10 says exact bytes in reports use exact byte comparison,
        // and a round trip is the cheapest way to know the bytes are the
        // transaction rather than a rendering of it.
        let fixture = fixture_bundle().expect("the fixture bundle builds");
        let census = positive_semantic_census().expect("the positive census builds");
        let case = census
            .iter()
            .find(|case| is_materializable(case))
            .expect("a sponsorless case exists");
        let vector = materialize(&fixture, case, &placeholder(case)).expect("it materializes");

        let decoded = TargetTransaction::decode(vector.bytes()).expect("the bytes decode");
        assert_eq!(decoded.encode(), vector.bytes());
        assert_eq!(decoded.weight(), vector.weight());
        assert_eq!(decoded.virtual_size(), vector.virtual_size());
        check_weight(fixture.target(), &decoded).expect("the fixture stays within the bound");
    }

    #[test]
    fn materialization_is_deterministic_to_the_byte() {
        // §18.1's last positive class, at the level this wave can reach:
        // no target has run, so the claim is about the construction.
        let fixture = fixture_bundle().expect("the fixture bundle builds");
        let census = positive_semantic_census().expect("the positive census builds");
        for case in census.iter().filter(|case| is_materializable(case)) {
            let first = materialize(&fixture, case, &placeholder(case)).expect("it materializes");
            let second = materialize(&fixture, case, &placeholder(case)).expect("it materializes");
            assert_eq!(first, second, "{:?} is not byte-stable", case.id());
        }
    }

    #[test]
    fn distinct_fixtures_produce_distinct_bytes() {
        let fixture = fixture_bundle().expect("the fixture bundle builds");
        let census = positive_semantic_census().expect("the positive census builds");
        let bytes: BTreeSet<Vec<u8>> = census
            .iter()
            .filter(|case| is_materializable(case))
            .map(|case| {
                materialize(&fixture, case, &placeholder(case))
                    .expect("it materializes")
                    .bytes()
                    .to_vec()
            })
            .collect();
        assert_eq!(
            bytes.len(),
            9,
            "two fixtures collided, so one vector would stand in for another"
        );
    }

    #[test]
    fn every_materialized_vector_carries_its_synthetic_disclaimers() {
        // §15.9: the funding is synthetic, and that has to travel with
        // the artifact. A vector whose disclaimer set emptied would be
        // claiming a provenance no ceremony established.
        let fixture = fixture_bundle().expect("the fixture bundle builds");
        let census = positive_semantic_census().expect("the positive census builds");
        for case in census.iter().filter(|case| is_materializable(case)) {
            let vector = materialize(&fixture, case, &placeholder(case)).expect("it materializes");
            let carried: BTreeSet<SyntheticDisclaimer> =
                vector.disclaimers().iter().copied().collect();
            let all: BTreeSet<SyntheticDisclaimer> =
                SyntheticDisclaimer::ALL.iter().copied().collect();
            assert_eq!(carried, all, "{:?} lost a disclaimer", case.id());
        }
    }

    #[test]
    fn the_placeholder_rule_is_injective_across_fixtures_and_members() {
        // Two fixtures sharing an outpoint would make one vector spend
        // another's coin, which is §18.3's first negative class arriving
        // by accident in the positive set.
        let census = positive_semantic_census().expect("the positive census builds");
        let mut seen = BTreeSet::new();
        for case in census.iter().filter(|case| is_materializable(case)) {
            for outpoint in placeholder(case).outpoints() {
                assert!(
                    seen.insert(*outpoint),
                    "the placeholder rule collided at {:?}",
                    case.id()
                );
            }
        }
    }

    #[test]
    fn funding_cut_for_one_vector_is_refused_by_another() {
        // The check that makes the funding record worth being a type.
        let fixture = fixture_bundle().expect("the fixture bundle builds");
        let census = positive_semantic_census().expect("the positive census builds");
        let mut sponsorless = census.iter().filter(|case| is_materializable(case));
        let first = sponsorless.next().expect("a first sponsorless case");
        let second = sponsorless.next().expect("a second sponsorless case");
        let error = materialize(&fixture, first, &placeholder(second))
            .expect_err("funding cut for another vector is refused");
        assert!(matches!(
            error,
            crate::error::VectorError::FundingNamesAnotherVector { .. }
        ));
    }

    #[test]
    fn a_vector_refuses_a_funding_record_of_the_wrong_size() {
        let census = positive_semantic_census().expect("the positive census builds");
        let case = census
            .iter()
            .find(|case| is_materializable(case))
            .expect("a sponsorless case exists");
        let id = vector_id(case);
        let error = AshFunding::new(id, Vec::new()).expect_err("no coins is not a funding record");
        assert!(matches!(
            error,
            crate::error::VectorError::FundingCardinalityMismatch { .. }
        ));
    }

    /// The first sponsored row this candidate has a program for.
    fn sponsored_case() -> CompactAshSemanticCase {
        positive_semantic_census()
            .expect("the positive census builds")
            .into_iter()
            .find(|case| super::needs_authorization(case) && super::has_candidate_program(case))
            .expect("a sponsored row with a program exists")
    }

    /// One sponsor coin at a version-zero key-hash program.
    ///
    /// The program class is the one the reviewed sponsor profile
    /// admits, which is what an executor holding a key would report; a
    /// coin at any other program is refused by the construction, and
    /// that refusal is its own test below.
    fn sponsor_coin(seed: u8, amount: u64) -> super::SponsorCoin {
        let mut bytes = [0_u8; 32];
        bytes[0] = 0xc0;
        bytes[1] = seed;
        let outpoint =
            Outpoint::new(Txid::from_internal(bytes), 0).expect("an admissible outpoint");
        let mut program = vec![0x00, 0x14];
        program.extend(std::iter::repeat_n(seed, 20));
        super::SponsorCoin::new(outpoint, amount, program)
    }

    /// Drive both passes with a stand-in for the executor.
    ///
    /// The authorization is a fixed stack of the admitted width bound
    /// to the exact bytes the request carried, which is the shape an
    /// executor's answer has. It authorizes nothing and no target sees
    /// it; what is under test here is the two-pass exchange, not
    /// whether a signature verifies.
    fn sponsored_vector(
        coins: &[super::SponsorCoin],
    ) -> Result<super::MaterializedTargetVector, crate::error::VectorError> {
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let case = sponsored_case();
        let funding = placeholder(&case);
        let tasks = super::sponsor_signing_requests(&bundle, &case, &funding, coins)?;
        let signatures = tasks
            .iter()
            .map(|task| {
                (
                    task.coin().outpoint(),
                    super::SponsorSignature::new(
                        task.request().transaction().to_vec(),
                        vec![vec![0x30; 71], vec![0x02; 33]],
                    ),
                )
            })
            .collect();
        super::materialize_sponsored(&bundle, &case, &funding, coins, &signatures)
    }

    #[test]
    fn a_sponsored_row_materializes_once_its_authorization_arrives() {
        // The whole point of the wave, at the construction level: the
        // row that could not be built now builds, and it builds into
        // the shape the fixture names rather than some near neighbour.
        let coins = [sponsor_coin(0x01, 100_000)];
        let vector = sponsored_vector(&coins).expect("the sponsored row materializes");
        assert_eq!(vector.id().sponsors(), 1);
        assert_eq!(vector.shape().sponsor_inputs(), 1);
        assert!(vector.shape().sponsored());
        assert_ne!(vector.bytes(), [] as [u8; 0]);

        // The successor is still the realization layer's own sum. A
        // sponsor pays the fee and contributes nothing to the protocol
        // object, so its arrival must not move this number.
        let case = sponsored_case();
        assert_eq!(
            vector.settled_successor(),
            case.expected().successor().1.get()
        );
    }

    #[test]
    fn the_two_passes_agree_and_the_second_is_what_carries_the_authorization() {
        // The soundness condition for running the construction twice.
        // The request the first pass produced states the bytes an
        // authorization must be about; the second pass settles the same
        // bytes, which is why the authorization still applies. If the
        // two ever diverged, the binding check below would fire instead
        // of this passing.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let case = sponsored_case();
        let funding = placeholder(&case);
        let coins = [sponsor_coin(0x02, 100_000)];
        let tasks = super::sponsor_signing_requests(&bundle, &case, &funding, &coins)
            .expect("the requests are collected");
        assert_eq!(tasks.len(), 1, "one member, one authorization");
        assert_eq!(tasks[0].coin().outpoint(), coins[0].outpoint());

        // The request is about the input after the ASH family, which is
        // where the suffix starts.
        assert_eq!(usize::from(tasks[0].request().input()), case.ash_inputs());

        let vector = sponsored_vector(&coins).expect("it materializes");
        // The authorized transaction is longer than the bytes that were
        // signed, because the witness it carries was empty at signing
        // time. That is the ordering §15.8 asks for, visible in the
        // sizes.
        assert!(vector.bytes().len() > tasks[0].request().transaction().len());
    }

    #[test]
    fn an_authorization_over_other_bytes_is_refused() {
        // The check that makes the echo worth carrying. An executor
        // that authorized some other transaction returns some other
        // bytes, and the construction refuses rather than attaching the
        // stack to a transaction it was not produced for.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let case = sponsored_case();
        let funding = placeholder(&case);
        let coins = [sponsor_coin(0x03, 100_000)];
        let tasks = super::sponsor_signing_requests(&bundle, &case, &funding, &coins)
            .expect("the requests are collected");

        let mut wrong = tasks[0].request().transaction().to_vec();
        wrong.push(0x00);
        let signatures = std::iter::once((
            tasks[0].coin().outpoint(),
            super::SponsorSignature::new(wrong, vec![vec![0x30; 71], vec![0x02; 33]]),
        ))
        .collect();

        let error = super::materialize_sponsored(&bundle, &case, &funding, &coins, &signatures)
            .expect_err("an authorization bound to other bytes is refused");
        assert!(
            matches!(
                error,
                crate::error::VectorError::TargetMaterializationFailed {
                    cause: transaction::TransactionRefusal::SponsorSignatureBindingMismatch(_),
                    ..
                }
            ),
            "refused with {error:?}"
        );
    }

    #[test]
    fn a_sponsored_row_with_no_authorization_is_refused() {
        // An absent authorization is not a transaction with an empty
        // witness. Nothing may stand in for it.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let case = sponsored_case();
        let funding = placeholder(&case);
        let coins = [sponsor_coin(0x04, 100_000)];
        let error =
            super::materialize_sponsored(&bundle, &case, &funding, &coins, &BTreeMap::new())
                .expect_err("no authorization is not a construction");
        assert!(
            matches!(
                error,
                crate::error::VectorError::TargetMaterializationFailed {
                    cause: transaction::TransactionRefusal::SponsorSignatureMissing(_),
                    ..
                }
            ),
            "refused with {error:?}"
        );
    }

    #[test]
    fn a_sponsor_coin_at_a_program_the_profile_does_not_admit_is_refused() {
        // The executor chooses the program, and the construction still
        // checks it. A coin at a taproot program cannot be spent by the
        // two-item stack the reviewed profile expects, and building
        // against one would produce a transaction refused for a reason
        // the row is not about.
        let outpoint = {
            let mut bytes = [0_u8; 32];
            bytes[0] = 0xc5;
            Outpoint::new(Txid::from_internal(bytes), 0).expect("an admissible outpoint")
        };
        let mut program = vec![0x51, 0x20];
        program.extend(std::iter::repeat_n(0x05, 32));
        let coins = [super::SponsorCoin::new(outpoint, 100_000, program)];

        let error = sponsored_vector(&coins).expect_err("the program class is not admitted");
        assert!(
            matches!(
                error,
                crate::error::VectorError::TargetMaterializationFailed {
                    cause: transaction::TransactionRefusal::SponsorProgramClassNotAdmitted(_),
                    ..
                }
            ),
            "refused with {error:?}"
        );
    }

    #[test]
    fn a_sponsored_materialization_is_deterministic_to_the_byte() {
        // §18.1's byte-stability class, for the sponsored half. The
        // authorization is an input here rather than something produced
        // twice, which is exactly the claim: given the same answers, the
        // construction settles the same transaction.
        let coins = [sponsor_coin(0x06, 100_000)];
        let first = sponsored_vector(&coins).expect("it materializes");
        let second = sponsored_vector(&coins).expect("it materializes");
        assert_eq!(first, second);
    }

    #[test]
    fn a_row_offered_the_wrong_number_of_sponsor_coins_is_refused() {
        // The row states how many members its world has. Building it
        // against a different number would settle a shape the fixture
        // never named, and the successor would be right by accident.
        let coins = [sponsor_coin(0x07, 50_000), sponsor_coin(0x08, 50_000)];
        let error = sponsored_vector(&coins).expect_err("two coins is not this row's region");
        assert!(
            matches!(
                error,
                crate::error::VectorError::MaterializedShapeMismatch(_)
            ),
            "refused with {error:?}"
        );
    }
}
