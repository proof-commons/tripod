//! The confidential-transaction fixture language and conservation matrix.
//!
//! Guide 11 §8 asks for three things that are easy to conflate and must
//! not be: a way to *state* a confidential transaction as test data
//! (§8.1–8.2), a vocabulary that says *where* an execution ended up
//! (§8.3), and a matrix of conservation cases whose expectations were
//! written before any target was asked (§8.4). This module owns the first
//! and third; the second is on the wire, in
//! [`crate::protocol::ObservedOutcomeLayer`], because it is what an
//! executor reports rather than what a fixture states.
//!
//! # Everything here is public disposable test data
//!
//! Every blinding factor, nonce seed, and range-proof seed in this module
//! is a fixed public constant with a documented recipe. None of it
//! authorizes anything, none derives from production material, and all of
//! it is destroyed with the disposable chain it is used on
//! `(´[ADR015-rule:security:test-material]´)`. Guide 11 §1.7 requires
//! exactly this: the guide introduces no production secret, and no
//! command in this package accepts one.
//!
//! # Why the representation is typed rather than raw bytes
//!
//! A confidential output could be stated as the bytes it serializes to,
//! and that would be shorter. It would also be unreadable: a reviewer
//! could not tell a fixture that means "ten units blinded by this factor"
//! from one that means "these thirty-three bytes", and the two fail
//! differently. A typed representation states the *intent*, and the
//! materializer's job is to produce bytes that match it — which is a
//! claim a run can then check, rather than a tautology.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// How a value is represented in one test fixture.
///
/// This is Guide 11 §8.1's shape. The confidential arm carries every
/// random input explicitly, because §8.2 requires that the fixture state
/// all of its own randomness rather than letting a materializer invent
/// some: a fixture whose blinders came from somewhere else cannot be
/// compared with anything, and two runs of it are not the same case.
///
/// Whether the *materializer* can honour the stated randomness is a
/// separate question, and on this target it cannot — see
/// [`DeterminismLevel`]. The fields stay explicit anyway: what a fixture
/// states is what a reader can check the run against, and dropping them
/// because one materializer ignores them would delete the statement
/// rather than the limitation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "representation")]
#[non_exhaustive]
pub enum TestValueRepresentation {
    /// An amount carried in the clear.
    Explicit {
        /// The amount, in the target's smallest unit.
        amount: u64,
    },
    /// An amount carried as a commitment.
    Confidential {
        /// The amount the commitment is to.
        amount: u64,
        /// The scalar blinding the asset generator.
        asset_blinding_factor: [u8; 32],
        /// The scalar blinding the value commitment.
        value_blinding_factor: [u8; 32],
        /// The seed the output's nonce is derived from.
        nonce_seed: [u8; 32],
        /// The seed the range proof is generated from.
        rangeproof_seed: [u8; 32],
    },
}

impl TestValueRepresentation {
    /// The amount, whichever representation carries it.
    ///
    /// A confidential amount is still an amount the fixture states; it is
    /// the *target* that cannot see it, not the test.
    #[must_use]
    pub const fn amount(&self) -> u64 {
        match self {
            Self::Explicit { amount } | Self::Confidential { amount, .. } => *amount,
        }
    }

    /// Whether this value is carried as a commitment.
    #[must_use]
    pub const fn is_confidential(&self) -> bool {
        matches!(self, Self::Confidential { .. })
    }

    /// One confidential value, with every seed derived from one label.
    ///
    /// The recipe is stated in [`test_scalar`], and it is a recipe rather
    /// than a table so that a reader can recompute any factor in this
    /// module from its label without consulting a golden file.
    #[must_use]
    pub fn confidential(amount: u64, label: &str) -> Self {
        Self::Confidential {
            amount,
            asset_blinding_factor: test_scalar(label, ROLE_ASSET_BLINDER),
            value_blinding_factor: test_scalar(label, ROLE_VALUE_BLINDER),
            nonce_seed: test_scalar(label, ROLE_NONCE),
            rangeproof_seed: test_scalar(label, ROLE_RANGEPROOF),
        }
    }
}

/// How an asset is represented in one test fixture.
///
/// Guide 11 §8.1 asks for "a corresponding asset representation stating
/// whether the asset is explicit or confidential". It is a separate
/// choice from the value's: a transaction may carry an explicit asset
/// with a confidential amount, and the wrong-generator row of the matrix
/// depends on being able to say so.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "representation")]
#[non_exhaustive]
pub enum TestAssetRepresentation {
    /// The asset identifier appears in the clear.
    Explicit {
        /// The asset identifier.
        asset_id: [u8; 32],
    },
    /// The asset appears as a blinded generator.
    Confidential {
        /// The asset identifier the generator derives from.
        asset_id: [u8; 32],
        /// The scalar blinding that generator.
        asset_blinding_factor: [u8; 32],
    },
}

impl TestAssetRepresentation {
    /// The asset identifier, whichever representation carries it.
    #[must_use]
    pub const fn asset_id(&self) -> &[u8; 32] {
        match self {
            Self::Explicit { asset_id } | Self::Confidential { asset_id, .. } => asset_id,
        }
    }

    /// Whether this asset is carried as a blinded generator.
    #[must_use]
    pub const fn is_confidential(&self) -> bool {
        matches!(self, Self::Confidential { .. })
    }
}

/// One stated value together with the asset it is denominated in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TestAmount {
    /// How the amount is represented.
    pub value: TestValueRepresentation,
    /// How the asset is represented.
    pub asset: TestAssetRepresentation,
}

/// The deliberate defect one matrix row introduces.
///
/// # Why the defect is named rather than pre-serialized
///
/// A malformed range proof could be stated as the exact bytes to write,
/// and then the fixture would be pinned to one materializer's output. It
/// is stated as an *operation* instead — "flip one bit of the range proof
/// the materializer produced" — because the property under test is that
/// the target refuses a proof that does not verify, and that property is
/// the same whichever valid proof was corrupted.
///
/// A run records the bytes it actually produced, so the row remains
/// reproducible as evidence even though it is not reproducible as bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ConservationDefect {
    /// No defect: the row is expected to stand.
    None,
    /// The outputs exceed the inputs by exactly one unit.
    OneUnitImbalance,
    /// The amounts are right and the blinding factors do not sum to zero.
    ///
    /// Produced by declaring an input blinder other than the one the
    /// consumed output actually carries, so the materializer balances
    /// against a scalar the chain does not agree with.
    WrongBlinderSum,
    /// One bit of a range proof is flipped.
    MalformedRangeProof,
    /// One bit of a surjection proof is flipped.
    MalformedSurjectionProof,
    /// An explicit output names an asset other than the one consumed.
    WrongExplicitAsset,
    /// An output's commitment is copied from an output of another asset.
    CopiedCommitmentFromOtherAsset,
    /// A confidential output is present that the stated output set omits.
    HiddenConfidentialOutput,
}

impl ConservationDefect {
    /// Whether this defect is expected to leave the transaction valid.
    #[must_use]
    pub const fn is_benign(&self) -> bool {
        matches!(self, Self::None)
    }
}

/// Why a row is stated but not executed.
///
/// A deferred row is not a passing row and not a failing one. It is a
/// case whose materialization this wave established it cannot build, and
/// saying so by name is the whole point: an absent row reads as an
/// oversight, and a row silently recorded as passing reads as evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RowDeferral {
    /// The row needs a public-committed representation, whose candidate
    /// support Guide 11 leaves open and whose blockers Wave 5 recorded.
    ///
    /// Guide 11 §8.4 states these rows as "accept if candidate supports",
    /// which is a conditional on a candidate none of which is selected.
    /// Executing the row would mean choosing one, and choosing one is
    /// §9–§11's question rather than this wave's.
    PublicCommittedCandidateUndecided,
}

/// An obligation consensus does not discharge.
///
/// # Why a row can carry a defect and still expect acceptance
///
/// Guide 11 §8.4 states the hidden-confidential-output row as "closure
/// reject **where claimed**", and the conditional is load-bearing. A
/// confidential output absorbing value is a perfectly valid transaction:
/// consensus checks that the commitments balance, and a hidden output
/// makes them balance. Nothing at the consensus layer is violated.
///
/// What such a transaction violates is an *output closure* property — the
/// claim that the stated output set is the whole output set — and that
/// claim belongs to a candidate relation, none of which this wave
/// selects. Writing the row's expectation as a consensus rejection would
/// therefore have been wrong, and a run would have "failed" against an
/// expectation the target never owed.
///
/// So the row expects acceptance at consensus and records the obligation
/// by name. That is the finding: value hiding is not policed by
/// conservation, and any candidate claiming disclosure-completeness has
/// to police it itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ClosureObligation {
    /// Consensus admits an unstated confidential output that absorbs
    /// value; only an output-closure relation refuses it.
    HiddenValueNotPolicedByConsensus,
}

/// One row of the Guide 11 §8.4 conservation matrix.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConservationRow {
    /// The row's position in the matrix, and its name.
    pub id: ConservationRowId,
    /// What the transaction consumes.
    pub inputs: Vec<TestAmount>,
    /// What the transaction is stated to create, fee aside.
    pub outputs: Vec<TestAmount>,
    /// The deliberate defect, where the row carries one.
    pub defect: ConservationDefect,
    /// Where the row's author expects the execution to end up.
    pub expected_layer: ExpectedOutcomeLayer,
    /// Why the row is not executed, where it is not.
    pub deferral: Option<RowDeferral>,
    /// The obligation consensus leaves undischarged, where the row's
    /// defect is one consensus does not police.
    pub closure_obligation: Option<ClosureObligation>,
}

impl ConservationRow {
    /// Whether this row is executed against a target at all.
    #[must_use]
    pub const fn is_executed(&self) -> bool {
        self.deferral.is_none()
    }

    /// Whether any stated value or asset is confidential.
    #[must_use]
    pub fn involves_confidential_value(&self) -> bool {
        self.inputs
            .iter()
            .chain(self.outputs.iter())
            .any(|amount| amount.value.is_confidential() || amount.asset.is_confidential())
    }
}

/// One row's identity.
///
/// The ordinal is the row's position in Guide 11 §8.4's own table, so a
/// reader can line the report up against the guide without matching
/// prose. The name is what a report row is read by.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConservationRowId {
    /// The row's position in the guide's table, counting from one.
    pub ordinal: u32,
    /// The row's name.
    pub name: String,
}

impl std::fmt::Display for ConservationRowId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:02}::{}", self.ordinal, self.name)
    }
}

/// Where a fixture's author expects one execution to end up.
///
/// This is the fixture's side of Guide 11 §8.3. The executor's side is
/// [`crate::protocol::ObservedOutcomeLayer`], and the two are separate
/// types on purpose: an expectation is written by a person before a run,
/// an observation is produced by an adapter during one, and a single type
/// used for both invites the comparison that reads one as the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExpectedOutcomeLayer {
    /// The target is expected to accept the transaction.
    Accepted,
    /// The target is expected to refuse it before any script runs.
    ConsensusRejectionBeforeScript,
    /// The target is expected to refuse it while running the script path.
    ScriptPathRejection,
    /// The target is expected to relay-refuse an otherwise valid
    /// transaction.
    RelayPolicyRejection,
}

/// How reproducible a materialized transaction is.
///
/// # The distinction this wave had to make
///
/// Guide 11 §8.2 requires that equal explicit inputs yield equal
/// transaction bytes. That requirement is met at the level a fixture can
/// control and not at the level of the bytes, and the difference is a
/// property of the target's own interfaces rather than of this harness.
///
/// Blinding on this target happens inside the node. `BlindTransaction`
/// draws every output blinding factor from `GetStrongRandBytes` and
/// generates a fresh ephemeral nonce key with `MakeNewKey`, and no RPC on
/// the path — `rawblindrawtransaction` included — takes a seed. The
/// upstream Python functional framework offers no Pedersen commitment,
/// range proof, or surjection proof of its own, so there is no
/// out-of-node materializer to substitute. Two runs of the same fixture
/// therefore produce different bytes, and this was confirmed both by
/// reading the source and by blinding one identical raw transaction twice
/// and comparing.
///
/// So the honest statement is [`Self::FixtureInputsOnly`]: the fixture's
/// inputs are fully determined and reproducible, the bytes are recorded
/// per run, and byte-level reproducibility is reported as not achievable
/// through this materializer. That is a finding about the target, not a
/// failure of the fixture language, and it is typed so that a later
/// materializer which *can* seed its blinding reports
/// [`Self::MaterializedBytes`] and the change is visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DeterminismLevel {
    /// Equal fixture inputs yield equal fixture bytes, and the
    /// materialized transaction bytes vary between runs.
    FixtureInputsOnly,
    /// Equal fixture inputs yield equal materialized transaction bytes.
    MaterializedBytes,
}

// The role labels the seed recipe separates factors by. A single label
// must not yield the same scalar for two different roles: a fixture whose
// value blinder equalled its asset blinder would be a weaker case wearing
// a general one's name.
const ROLE_ASSET_BLINDER: &str = "asset-blinder";
const ROLE_VALUE_BLINDER: &str = "value-blinder";
const ROLE_NONCE: &str = "nonce";
const ROLE_RANGEPROOF: &str = "rangeproof";

/// One 32-byte public test scalar, from a label and a role.
///
/// # The recipe, stated rather than tabulated
///
/// The scalar is `SHA-256("tripod/guide11/ct-fixture/" || role
/// || "/" || label)`. Any reader can recompute it, which is what makes
/// the fixture data checkable without a golden file, and the domain
/// prefix keeps these values from colliding with any other hash this
/// project derives.
///
/// The result is reduced only in the sense that a scalar at or above the
/// group order would be refused by the target; no fixture in this module
/// hits that, and the matrix does not depend on one. This is public test
/// data and nothing more: it authorizes nothing, and it exists only for
/// disposable development chains `(´[ADR015-rule:security:test-material]´)`.
#[must_use]
pub fn test_scalar(label: &str, role: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(b"tripod/guide11/ct-fixture/");
    hasher.update(role.as_bytes());
    hasher.update(b"/");
    hasher.update(label.as_bytes());
    hasher.finalize().into()
}

/// The policy asset of the disposable chain, as a fixture states it.
///
/// A fixture cannot know which asset a development network issues — that
/// is the network's own fact, and the reviewed adapter already supplies
/// it for primitive fixtures on the same reasoning. The identifier here
/// is the all-zero placeholder meaning "the chain's own policy asset",
/// and the materializer substitutes what it observed.
pub const CHAIN_POLICY_ASSET: [u8; 32] = [0_u8; 32];

/// An asset identifier that is deliberately not the chain's.
///
/// Used by the wrong-asset and copied-commitment rows. It is a public
/// constant with no issuance behind it, which is exactly what makes those
/// rows refusals rather than transfers.
pub const FOREIGN_TEST_ASSET: [u8; 32] = [0x5a_u8; 32];

/// One explicit amount of the chain's policy asset.
const fn explicit(amount: u64) -> TestAmount {
    TestAmount {
        value: TestValueRepresentation::Explicit { amount },
        asset: TestAssetRepresentation::Explicit {
            asset_id: CHAIN_POLICY_ASSET,
        },
    }
}

/// One confidential amount of the chain's policy asset.
fn confidential(amount: u64, label: &str) -> TestAmount {
    TestAmount {
        value: TestValueRepresentation::confidential(amount, label),
        asset: TestAssetRepresentation::Confidential {
            asset_id: CHAIN_POLICY_ASSET,
            asset_blinding_factor: test_scalar(label, ROLE_ASSET_BLINDER),
        },
    }
}

/// The canonical Guide 11 §8.4 conservation matrix.
///
/// # Every expectation is written here, before any target is asked
///
/// The rows carry their expected layer as data, and the executor is never
/// sent it: a conservation request carries the row's construction and
/// nothing about what should happen to it, exactly as protocol revision 3
/// requires of every other request shape
/// `(´[PLAN-rule:guide11-exec:request-subject]´)`.
///
/// # Twelve rows, eleven executed
///
/// The guide's table has twelve rows. Two of them name a *public
/// committed* output, and the guide states their expectation
/// conditionally — "accept if candidate supports" — against a candidate
/// that §9–§11 have not selected. One of those two, "several confidential
/// inputs → public committed output", is executed here in the form the
/// guide's §10.2 discusses under "Several private inputs → Explicit",
/// because that form needs no undecided candidate and is the part of the
/// row this wave can honestly answer; the public-committed form of it is
/// recorded alongside as deferred. The other is deferred outright.
///
/// So: eleven rows are executed and one is deferred, and the deferral is
/// a typed row in the matrix rather than an absence.
#[must_use]
// One literal per row, in the guide's own order. Splitting this to satisfy
// a line count would scatter twelve statements a reader checks against one
// table across several functions, which costs more than it saves.
#[allow(clippy::too_many_lines)]
pub fn canonical_conservation_matrix() -> Vec<ConservationRow> {
    // A single funding amount keeps every row's arithmetic legible: a
    // reader checking a row does not also have to check that its totals
    // were chosen consistently.
    const UNIT: u64 = 10_000_000;

    vec![
        ConservationRow {
            id: row(1, "explicit-to-explicit-balanced"),
            inputs: vec![explicit(UNIT)],
            outputs: vec![explicit(UNIT)],
            defect: ConservationDefect::None,
            expected_layer: ExpectedOutcomeLayer::Accepted,
            deferral: None,
            closure_obligation: None,
        },
        ConservationRow {
            id: row(2, "confidential-to-confidential-balanced"),
            inputs: vec![confidential(UNIT, "row2-input")],
            outputs: vec![confidential(UNIT, "row2-output")],
            defect: ConservationDefect::None,
            expected_layer: ExpectedOutcomeLayer::Accepted,
            deferral: None,
            closure_obligation: None,
        },
        ConservationRow {
            id: row(3, "confidential-to-public-committed-balanced"),
            inputs: vec![confidential(UNIT, "row3-input")],
            outputs: vec![confidential(UNIT, "row3-output")],
            defect: ConservationDefect::None,
            expected_layer: ExpectedOutcomeLayer::Accepted,
            deferral: Some(RowDeferral::PublicCommittedCandidateUndecided),
            closure_obligation: None,
        },
        ConservationRow {
            id: row(4, "confidential-to-explicit-with-private-change"),
            inputs: vec![confidential(UNIT, "row4-input")],
            outputs: vec![
                explicit(UNIT / 2),
                confidential(UNIT / 2, "row4-private-change"),
            ],
            defect: ConservationDefect::None,
            expected_layer: ExpectedOutcomeLayer::Accepted,
            deferral: None,
            closure_obligation: None,
        },
        ConservationRow {
            id: row(5, "several-confidential-to-one-output"),
            inputs: vec![
                confidential(UNIT, "row5-input-a"),
                confidential(UNIT, "row5-input-b"),
            ],
            outputs: vec![explicit(UNIT * 2)],
            defect: ConservationDefect::None,
            expected_layer: ExpectedOutcomeLayer::Accepted,
            deferral: None,
            closure_obligation: None,
        },
        ConservationRow {
            id: row(6, "one-unit-semantic-imbalance"),
            inputs: vec![explicit(UNIT)],
            outputs: vec![explicit(UNIT + 1)],
            defect: ConservationDefect::OneUnitImbalance,
            expected_layer: ExpectedOutcomeLayer::ConsensusRejectionBeforeScript,
            deferral: None,
            closure_obligation: None,
        },
        ConservationRow {
            id: row(7, "correct-amounts-wrong-blinding-balance"),
            inputs: vec![confidential(UNIT, "row7-input")],
            outputs: vec![confidential(UNIT, "row7-output")],
            defect: ConservationDefect::WrongBlinderSum,
            expected_layer: ExpectedOutcomeLayer::ConsensusRejectionBeforeScript,
            deferral: None,
            closure_obligation: None,
        },
        ConservationRow {
            id: row(8, "malformed-rangeproof"),
            inputs: vec![confidential(UNIT, "row8-input")],
            outputs: vec![confidential(UNIT, "row8-output")],
            defect: ConservationDefect::MalformedRangeProof,
            expected_layer: ExpectedOutcomeLayer::ConsensusRejectionBeforeScript,
            deferral: None,
            closure_obligation: None,
        },
        ConservationRow {
            id: row(9, "malformed-surjection-proof"),
            inputs: vec![confidential(UNIT, "row9-input")],
            outputs: vec![confidential(UNIT, "row9-output")],
            defect: ConservationDefect::MalformedSurjectionProof,
            expected_layer: ExpectedOutcomeLayer::ConsensusRejectionBeforeScript,
            deferral: None,
            closure_obligation: None,
        },
        ConservationRow {
            id: row(10, "wrong-explicit-asset-generator"),
            inputs: vec![explicit(UNIT)],
            outputs: vec![TestAmount {
                value: TestValueRepresentation::Explicit { amount: UNIT },
                asset: TestAssetRepresentation::Explicit {
                    asset_id: FOREIGN_TEST_ASSET,
                },
            }],
            defect: ConservationDefect::WrongExplicitAsset,
            expected_layer: ExpectedOutcomeLayer::ConsensusRejectionBeforeScript,
            deferral: None,
            closure_obligation: None,
        },
        ConservationRow {
            id: row(11, "copied-commitment-from-another-asset"),
            inputs: vec![confidential(UNIT, "row11-input")],
            outputs: vec![confidential(UNIT, "row11-output")],
            defect: ConservationDefect::CopiedCommitmentFromOtherAsset,
            expected_layer: ExpectedOutcomeLayer::ConsensusRejectionBeforeScript,
            deferral: None,
            closure_obligation: None,
        },
        ConservationRow {
            id: row(12, "hidden-confidential-output"),
            inputs: vec![confidential(UNIT, "row12-input")],
            outputs: vec![explicit(UNIT / 2)],
            defect: ConservationDefect::HiddenConfidentialOutput,
            // Not a consensus rejection, and the guide does not claim one:
            // §8.4 states this row as "closure reject *where claimed*". A
            // hidden confidential output makes the commitments balance, so
            // conservation is satisfied and consensus has nothing to
            // refuse. The obligation belongs to a closure relation, and no
            // candidate that would own it is selected in this wave.
            expected_layer: ExpectedOutcomeLayer::Accepted,
            deferral: None,
            closure_obligation: Some(ClosureObligation::HiddenValueNotPolicedByConsensus),
        },
    ]
}

/// Exactly what an executor is asked to materialize and judge.
///
/// # The answer stays with the harness
///
/// The subject carries the transaction to build and the defect to
/// introduce. It does not carry [`ConservationRow::expected_layer`], and
/// it never will: under protocol revision 3 a request carries the
/// execution subject alone, so there is no expectation for an executor to
/// consult, echo, or drift toward
/// `(´[PLAN-rule:guide11-exec:request-subject]´)`.
///
/// This matters more here than for a primitive case. A conservation row's
/// whole content is *where* the target refused, and an adapter that knew
/// which layer was wanted could report that layer for a transaction that
/// never reached it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConservationSubject {
    /// What the transaction consumes.
    pub inputs: Vec<TestAmount>,
    /// What the transaction creates, fee aside.
    pub outputs: Vec<TestAmount>,
    /// The deliberate defect to introduce.
    pub defect: ConservationDefect,
}

impl ConservationRow {
    /// The part of this row an executor is allowed to see.
    #[must_use]
    pub fn subject(&self) -> ConservationSubject {
        ConservationSubject {
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            defect: self.defect,
        }
    }
}

/// Names one row.
fn row(ordinal: u32, name: &str) -> ConservationRowId {
    ConservationRowId {
        ordinal,
        name: name.to_owned(),
    }
}

/// Every distinct defect the canonical matrix exercises.
///
/// Used by the report to state which defects a run actually reached,
/// rather than which ones the matrix names.
#[must_use]
pub fn exercised_defects(rows: &[ConservationRow]) -> BTreeSet<ConservationDefect> {
    rows.iter()
        .filter(|row| row.is_executed())
        .map(|row| row.defect)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_matrix_has_the_guides_twelve_rows() {
        let matrix = canonical_conservation_matrix();
        assert_eq!(matrix.len(), 12);
        for (index, entry) in matrix.iter().enumerate() {
            let expected = u32::try_from(index + 1).expect("a twelve-row index fits");
            assert_eq!(entry.id.ordinal, expected);
        }
    }

    #[test]
    fn exactly_one_row_is_deferred_and_says_why() {
        let matrix = canonical_conservation_matrix();
        let deferred: Vec<_> = matrix.iter().filter(|row| !row.is_executed()).collect();
        assert_eq!(deferred.len(), 1);
        assert_eq!(
            deferred[0].deferral,
            Some(RowDeferral::PublicCommittedCandidateUndecided)
        );
    }

    #[test]
    fn equal_fixture_inputs_yield_equal_fixture_bytes() {
        // Guide 11 section 8.2, at the level this materializer can honour:
        // the fixture itself is a pure function of its stated inputs, so
        // two constructions of the matrix are byte-identical.
        let first =
            serde_json::to_vec(&canonical_conservation_matrix()).expect("the matrix serializes");
        let second =
            serde_json::to_vec(&canonical_conservation_matrix()).expect("the matrix serializes");
        assert_eq!(first, second);
    }

    #[test]
    fn a_confidential_value_separates_its_four_seeds() {
        // Four roles under one label must not collide: a fixture whose
        // value blinder equalled its asset blinder would be a weaker case
        // wearing a general one's name.
        let value = TestValueRepresentation::confidential(7, "collision-probe");
        let TestValueRepresentation::Confidential {
            asset_blinding_factor,
            value_blinding_factor,
            nonce_seed,
            rangeproof_seed,
            ..
        } = value
        else {
            panic!("the constructor builds a confidential value");
        };
        let seeds = BTreeSet::from([
            asset_blinding_factor,
            value_blinding_factor,
            nonce_seed,
            rangeproof_seed,
        ]);
        assert_eq!(seeds.len(), 4);
    }

    #[test]
    fn the_seed_recipe_is_reproducible_and_label_dependent() {
        assert_eq!(
            test_scalar("row2-input", ROLE_VALUE_BLINDER),
            test_scalar("row2-input", ROLE_VALUE_BLINDER)
        );
        assert_ne!(
            test_scalar("row2-input", ROLE_VALUE_BLINDER),
            test_scalar("row2-output", ROLE_VALUE_BLINDER)
        );
    }

    #[test]
    fn every_benign_row_is_expected_to_be_accepted() {
        // The two must not drift apart: a row carrying no defect and
        // expecting a rejection would be stating a target fact nothing in
        // the row explains.
        for entry in canonical_conservation_matrix() {
            if entry.defect.is_benign() {
                assert_eq!(entry.expected_layer, ExpectedOutcomeLayer::Accepted);
                assert_eq!(entry.closure_obligation, None);
            } else if entry.expected_layer == ExpectedOutcomeLayer::Accepted {
                // The one admitted exception, and it has to say why: a
                // defect consensus does not police names the obligation
                // that does. Without this the row would read as a target
                // fact nothing in the row explains.
                assert!(entry.closure_obligation.is_some());
            }
        }
    }

    #[test]
    fn only_the_hidden_output_row_carries_a_closure_obligation() {
        let matrix = canonical_conservation_matrix();
        let carrying: Vec<_> = matrix
            .iter()
            .filter(|row| row.closure_obligation.is_some())
            .collect();
        assert_eq!(carrying.len(), 1);
        assert_eq!(
            carrying[0].defect,
            ConservationDefect::HiddenConfidentialOutput
        );
    }

    #[test]
    fn the_confidential_rows_are_actually_confidential() {
        let matrix = canonical_conservation_matrix();
        // Rows 2, 3, 4, 5, 7, 8, 9, 11, 12 carry confidential value.
        assert_eq!(
            matrix
                .iter()
                .filter(|row| row.involves_confidential_value())
                .count(),
            9
        );
    }
}
