//! The two adapters the transaction-wide confidential materializer is
//! injected with, and the only place both sides are reachable.
//!
//! # Why the adapters live here and nowhere else
//!
//! The construction package declares two traits and implements neither,
//! because it may not depend on the conformance package: that package is
//! the independent oracle its output is compared against, and the absence
//! of the edge is what keeps the comparison from being one opinion
//! wearing two hats. This library is the one that can see both, so this
//! is where the two are wired together.
//!
//! The wiring is a boundary widening and is recorded as one in this
//! package's own manifest, beside the executor boundary that was there
//! before. A wave that added the use statement and no note would have
//! broken the rule even though the code compiled.
//!
//! # The two origins, and why they are not interchangeable
//!
//! [`FirstPartyCommitmentCheck`] is the first-party bignum commitment
//! arithmetic over published constants. Its independence claim is
//! unqualified: it computes points, it never touches a node, and it is
//! the same arithmetic whether a chain exists or not.
//!
//! [`ReferenceConfidentialMaterializer`] is the bindings the target
//! itself vendors. Everything it produces is conformance-to-the-target's-
//! own-implementation evidence and no report may call it independent —
//! which is exactly why it is admissible as the CONSTRUCTION and
//! inadmissible as the check on it. The construction refuses a check
//! declaring this origin rather than leaving the rule to a reviewer, so
//! the two cannot be swapped by accident.

use target_elements_conformance::commitment_oracle::commitment::commitment;
use target_elements_conformance::confidential_fixture::solve_balancing_blinder;
use target_elements_conformance::confidential_oracles::ReferenceProofMaterial;
use transaction::bytes::{AssetId, COMMITMENT_BYTES};
use transaction::live_materialize::{
    CommitmentOrigin, ConfidentialProofMaterializer, IndependentCommitment,
    IndependentCommitmentCheck, MaterializedRangeproof, MaterializerCommitment, RangeproofRequest,
    SCALAR_BYTES,
};

/// The independent check, over the first-party bignum oracle.
///
/// The only admitted implementation of its trait. It is stateless because
/// the arithmetic is: the same three inputs produce the same point on
/// every call, in every process, forever.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FirstPartyCommitmentCheck;

impl FirstPartyCommitmentCheck {
    /// One check.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl IndependentCommitmentCheck for FirstPartyCommitmentCheck {
    fn origin(&self) -> CommitmentOrigin {
        CommitmentOrigin::FirstPartyBignumOracle
    }

    fn recompute(
        &self,
        explicit_asset: AssetId,
        semantic_amount: u64,
        value_blinder: &[u8; SCALAR_BYTES],
    ) -> Option<IndependentCommitment> {
        let bytes = commitment(explicit_asset.internal(), semantic_amount, value_blinder).ok()?;
        Some(IndependentCommitment::from_independent_recomputation(bytes))
    }

    fn solve_balancing_blinder(
        &self,
        input_blinder_sum: &[u8; SCALAR_BYTES],
        other_blinders: &[[u8; SCALAR_BYTES]],
    ) -> Option<[u8; SCALAR_BYTES]> {
        solve_balancing_blinder(input_blinder_sum, other_blinders)
    }
}

/// The construction's own proof material, through the reference
/// bindings.
///
/// Conformance evidence and not independence, which is stated by the
/// origin it declares rather than by this comment: a build that wired it
/// in as the independent check is refused by the construction, naming the
/// output the check was for.
#[derive(Debug, Default)]
pub struct ReferenceConfidentialMaterializer {
    material: ReferenceProofMaterial,
}

impl ReferenceConfidentialMaterializer {
    /// One materializer, with its own library context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            material: ReferenceProofMaterial::new(),
        }
    }
}

impl ConfidentialProofMaterializer for ReferenceConfidentialMaterializer {
    fn origin(&self) -> CommitmentOrigin {
        CommitmentOrigin::ReferenceImplementation
    }

    fn value_commitment(
        &self,
        explicit_asset: AssetId,
        semantic_amount: u64,
        value_blinder: &[u8; SCALAR_BYTES],
    ) -> Option<MaterializerCommitment> {
        let bytes = self.material.value_commitment(
            explicit_asset.internal(),
            semantic_amount,
            value_blinder,
        )?;
        Some(MaterializerCommitment::from_materializer(bytes))
    }

    fn nonce_commitment(&self, nonce_input: &[u8; SCALAR_BYTES]) -> Option<[u8; COMMITMENT_BYTES]> {
        self.material.nonce_commitment(nonce_input)
    }

    fn range_proof(&self, request: &RangeproofRequest<'_>) -> Option<MaterializedRangeproof> {
        let proof = self.material.range_proof(
            request.explicit_asset().internal(),
            request.semantic_amount(),
            request.value_blinder(),
            request.seed(),
            request.value_commitment().bytes(),
            request.output_program(),
        )?;
        Some(MaterializedRangeproof::new(
            proof,
            // Empty, always. The form this arc fixes pairs a confidential
            // value with an EXPLICIT asset, so the asset carries no
            // blinder, the generator is the asset's own, and there is
            // nothing for a surjection proof to relate.
            Vec::new(),
            *request.value_commitment().bytes(),
            request.explicit_asset(),
            request.output_program().to_vec(),
        ))
    }
}
