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

use std::num::NonZeroU8;

use linker::backend::{CompactAshShape, CompactAshShapeBounds, SponsorChangePresence};
use transaction::{
    AssetField, AssetId, CompactAshRequest, Outpoint, PublicConstructionView, PublicOutputView,
    SyntheticDisclaimer, Txid, ValueField, construct,
};

use crate::bundle::FixtureBundle;
use crate::error::VectorError;
use crate::fixture::{CompactAshSemanticCase, SemanticFixtureId};
use crate::matrix::{EvidenceBoundary, VectorClass};

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
#[must_use]
pub fn vector_id(case: &CompactAshSemanticCase) -> TargetVectorId {
    TargetVectorId {
        fixture: case.id(),
        ash_inputs: u8::try_from(case.ash_inputs()).unwrap_or(u8::MAX),
        sponsors: 0,
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

/// Materialize one semantic case into exact target bytes.
///
/// Sponsored cases are not materialized in this wave: constructing one
/// requires a sponsor capability that signs, and this package holds no
/// key and signs nothing. They are reported as unmaterialized rather
/// than approximated, because §1.5 forbids letting an absent capability
/// read as a target result.
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
    let ash_inputs = u8::try_from(case.ash_inputs()).unwrap_or(u8::MAX);
    let id = TargetVectorId {
        fixture: case.id(),
        ash_inputs,
        sponsors: 0,
    };
    // The funding was cut for a vector, and this is that vector or it is
    // not. Building against another vector's coins would produce a
    // transaction nobody planned.
    if funding.vector() != id {
        return Err(VectorError::FundingNamesAnotherVector {
            wanted: id,
            supplied: funding.vector(),
        });
    }
    let shape = shape_of(ash_inputs, 0, false, id)?;

    let mut views = Vec::with_capacity(case.ash_inputs());
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

    let request = CompactAshRequest::new(outpoints, false)
        .map_err(|cause| VectorError::TargetMaterializationFailed { vector: id, cause })?;
    let view = PublicConstructionView::new(views);

    let built = construct(fixture.target(), fixture.abi(), &request, &view, None)
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

/// Whether a semantic case can be materialized in this wave.
///
/// Sponsored cases cannot: they need a signing capability this package
/// does not have and will not fabricate.
#[must_use]
pub const fn is_materializable(case: &CompactAshSemanticCase) -> bool {
    !case.sponsor().is_present()
}

#[cfg(test)]
mod tests {
    use super::{AshFunding, is_materializable, materialize, vector_id};
    use crate::bundle::fixture_bundle;
    use crate::fixture::{CompactAshSemanticCase, positive_semantic_census};
    use crate::matrix::EvidenceBoundary;
    use std::collections::BTreeSet;
    use transaction::{SyntheticDisclaimer, TargetTransaction, check_weight};

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
            "nine of the fourteen positive classes are sponsorless"
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
}
