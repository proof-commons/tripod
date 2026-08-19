//! Public commitment vectors and the three-way comparison hook.
//!
//! A vector states its own inputs and the bytes the oracle predicts for
//! them. Nothing here reads a target observation as an expected value:
//! the comparison runs one way, from a stated expectation to an observed
//! string, and a disagreement is reported rather than absorbed.

use super::commitment::{CommitmentDefect, commitment};
use super::curve::PREFIXED_POINT_BYTES;
use super::generator::{GeneratorDefect, serialized_asset_generator};

/// One public commitment vector.
///
/// The expected fields are stated as bytes rather than computed on
/// construction, so a test pins them and [`verify`](Self::verify)
/// recomputes: a vector whose pinned bytes drift from the recipe fails
/// rather than quietly following it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicCommitmentVector {
    /// The asset identifier the generator is derived from.
    pub asset_id: [u8; 32],
    /// The amount the commitment carries.
    pub amount: u64,
    /// The blinding scalar, thirty-two big-endian bytes.
    pub blinding_factor: [u8; 32],
    /// The serialized generator the oracle predicts.
    pub expected_generator: Vec<u8>,
    /// The serialized commitment the oracle predicts.
    pub expected_commitment: Vec<u8>,
}

/// Why a vector's pinned bytes are not what the recipe produces.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum VectorDefect {
    /// The asset identifier does not yield a generator.
    Generator(GeneratorDefect),
    /// The inputs do not yield a commitment.
    Commitment(CommitmentDefect),
    /// The pinned generator is not the derived one.
    GeneratorMismatch(PointMismatch),
    /// The pinned commitment is not the constructed one.
    CommitmentMismatch(PointMismatch),
}

impl PublicCommitmentVector {
    /// Recomputes the vector and checks its pinned bytes.
    ///
    /// # Errors
    ///
    /// [`VectorDefect`] when the recipe refuses the inputs, or when a
    /// pinned field disagrees with what it produces.
    pub fn verify(&self) -> Result<(), VectorDefect> {
        let derived =
            serialized_asset_generator(&self.asset_id).map_err(VectorDefect::Generator)?;
        compare_points(&derived, &self.expected_generator)
            .map_err(VectorDefect::GeneratorMismatch)?;

        let constructed = commitment(&self.asset_id, self.amount, &self.blinding_factor)
            .map_err(VectorDefect::Commitment)?;
        compare_points(&constructed, &self.expected_commitment)
            .map_err(VectorDefect::CommitmentMismatch)
    }
}

/// How an observed point encoding differs from an expected one.
///
/// The prefix and the x coordinate are reported separately because they
/// fail for different reasons: a prefix-only difference is the two
/// parties disagreeing about which of two y values the same x names,
/// which is a convention fault, while an x difference is a different
/// point altogether.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PointMismatch {
    /// The observed string is not thirty-three bytes.
    Width {
        /// The width a canonical encoding has.
        expected: usize,
        /// The width observed.
        found: usize,
    },
    /// The x coordinates agree and the parity prefixes do not.
    ParityPrefix {
        /// The prefix the oracle predicts.
        expected: u8,
        /// The prefix observed.
        found: u8,
    },
    /// The x coordinates differ, so the points differ.
    Coordinate {
        /// The whole encoding the oracle predicts.
        expected: Vec<u8>,
        /// The whole encoding observed.
        found: Vec<u8>,
    },
}

/// Compares an expected point encoding against an observed one.
///
/// This is the hook the three-way comparison uses. The third leg — a
/// commitment read back out of the target by introspection — is supplied
/// by a later wave's native run; this function is what that run will
/// call, and it exists now so the comparison is stated before any
/// observation is available to shape it.
///
/// # Errors
///
/// [`PointMismatch`] describing how the two differ.
pub fn compare_points(expected: &[u8], observed: &[u8]) -> Result<(), PointMismatch> {
    if observed.len() != PREFIXED_POINT_BYTES {
        return Err(PointMismatch::Width {
            expected: PREFIXED_POINT_BYTES,
            found: observed.len(),
        });
    }
    if expected == observed {
        return Ok(());
    }
    if expected[1..] == observed[1..] {
        return Err(PointMismatch::ParityPrefix {
            expected: expected[0],
            found: observed[0],
        });
    }
    Err(PointMismatch::Coordinate {
        expected: expected.to_vec(),
        found: observed.to_vec(),
    })
}

/// Which party produced a commitment string being compared.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CommitmentSource {
    /// A transaction-construction library.
    ConstructionLibrary,
    /// The target, read back by introspection.
    TargetIntrospection,
}

/// One leg's disagreement with the oracle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegDisagreement {
    /// Which party produced the differing string.
    pub source: CommitmentSource,
    /// How it differs from the oracle's expectation.
    pub mismatch: PointMismatch,
}

/// The three-way comparison of §7.4.
///
/// The oracle's expectation is the subject; the other two legs are
/// optional because they arrive from separate runs. A leg that is absent
/// is reported as absent rather than as agreement, so a comparison that
/// never actually happened cannot read as one that passed.
#[derive(Clone, Debug, Default)]
pub struct ThreeWayComparison {
    /// The bytes a transaction-construction library produced.
    pub construction_library: Option<Vec<u8>>,
    /// The bytes the target reported by introspection.
    pub target_introspection: Option<Vec<u8>>,
}

/// What a three-way comparison established.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThreeWayOutcome {
    /// The legs that were supplied and agreed with the oracle.
    pub agreed: Vec<CommitmentSource>,
    /// The legs that were supplied and disagreed.
    pub disagreed: Vec<LegDisagreement>,
    /// The legs that were not supplied at all.
    pub absent: Vec<CommitmentSource>,
}

impl ThreeWayOutcome {
    /// Whether every supplied leg agreed.
    ///
    /// This is not a claim that the comparison was complete; read
    /// [`absent`](Self::absent) for that.
    #[must_use]
    pub const fn supplied_legs_agree(&self) -> bool {
        self.disagreed.is_empty()
    }

    /// Whether both legs were supplied and both agreed.
    #[must_use]
    pub const fn complete_and_agreeing(&self) -> bool {
        self.absent.is_empty() && self.disagreed.is_empty()
    }
}

impl ThreeWayComparison {
    /// Compares both legs against the oracle's expectation.
    #[must_use]
    pub fn against(&self, expected: &[u8]) -> ThreeWayOutcome {
        let mut agreed = Vec::new();
        let mut disagreed = Vec::new();
        let mut absent = Vec::new();

        for (source, observed) in [
            (
                CommitmentSource::ConstructionLibrary,
                self.construction_library.as_ref(),
            ),
            (
                CommitmentSource::TargetIntrospection,
                self.target_introspection.as_ref(),
            ),
        ] {
            match observed {
                None => absent.push(source),
                Some(observed) => match compare_points(expected, observed) {
                    Ok(()) => agreed.push(source),
                    Err(mismatch) => disagreed.push(LegDisagreement { source, mismatch }),
                },
            }
        }

        ThreeWayOutcome {
            agreed,
            disagreed,
            absent,
        }
    }
}
