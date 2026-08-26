//! The sponsor's own reserve coins, registered as a public fixture so
//! that a committed value has somewhere to be written down.
//!
//! # Why this case exists beside the predecessor's
//!
//! The predecessor fixture registers coins of a PROTOCOL asset that a
//! run issues, paid to programs a run chooses, and never spent. This one
//! registers coins of the chain's RESERVE, paid to a program the
//! EXECUTOR chooses, and it exists precisely so that one of them can be
//! spent. The two differ in every one of those, and folding them into
//! one case would mean a manifest whose asset, whose programs and whose
//! purpose were each two things at once.
//!
//! # What the caller knows and what it is told
//!
//! Both the reserve and the sponsor program are the executor's facts,
//! and both are learned rather than chosen: an earlier explicit
//! sponsor-funding step reports a coin, and the asset and the program on
//! that coin are what this manifest is built against. So a run cannot
//! register this fixture until it has asked the executor a question, and
//! that ordering is the same one the predecessor ceremony has for the
//! issued asset.
//!
//! The request that follows names neither. It carries the handle, the
//! digest, and the ordered programs, and the executor resolves them in
//! its own catalogue against the reserve IT reads off the chain. If the
//! two ever disagreed about which asset the reserve is, the digests
//! would not match and the step would refuse at the binding rather than
//! producing coins of an asset nobody meant.
//!
//! # Nothing here is an expectation about a chain
//!
//! This module states what to register and what to ask for. What came
//! back is read from the mined bytes.

use target_elements_conformance::confidential_fixture::{
    ConfidentialFixtureManifest, ConfidentialFixtureOutput, ConfidentialFixtureRegistry,
    FixtureDerivationProfile, FixtureOutputRole, FrozenConfidentialFixtureRegistry,
    MAX_PARITY_COUNTER, PublicDisposableTestMaterial, RegistrationRefusal, sponsor_reserve_handle,
};
use target_elements_conformance::protocol::{
    ConfidentialFixtureDigest, ConfidentialFundingBinding, ConfidentialFundingDestination,
    TargetConfidentialSponsorFundingSubject,
};

use crate::confidential_predecessor::selected_profiles;

/// The caller's own name for the confidential sponsor-funding step.
pub const FUND_SPONSOR_STEP: &str = "fund-confidential-sponsor-reserve";

/// The semantic amounts the sponsor's reserve coins carry, in fixed
/// order.
///
/// Public disposable test material on a chain the run creates and
/// destroys `(ADR-015 rule test-material)`. They are stated here and in
/// the executor's own catalogue, and the fixture digest is what detects
/// the two drifting apart.
///
/// # Why the first is the number it is
///
/// It is the offer of the with-change sponsored shape: the fee that
/// shape pays plus the change it takes back. The coin this fixture's
/// first output creates is the coin that shape spends, so the reserve
/// sub-equation closes on it or the target refuses the transaction. The
/// equation itself is held where the shape states it and is not restated
/// here; what is here is the funded number, bound to that one by a test.
///
/// # Why the second is the SAME number
///
/// So the pair witnesses the property a commitment is for. Two outputs
/// carrying equal amounts under different blinders have different
/// serialized commitments, and a reader who could recover an amount from
/// a commitment would find these two identical. They are not, and the
/// run says so from the mined bytes rather than from here.
pub const SPONSOR_RESERVE_AMOUNTS: [u64; 2] = [1_250, 1_250];

/// How many coins the case funds.
pub const SPONSOR_RESERVE_OUTPUTS: usize = SPONSOR_RESERVE_AMOUNTS.len();

/// The sponsor reserve manifest, against one reserve asset and one
/// executor program.
///
/// # Why both outputs pay ONE program
///
/// Because both are the executor's coins and there is nowhere else for
/// them to go. The second output exists to solve the blinder sum, not to
/// pay anybody: a coin paid to some other program would be reserve value
/// this run could never account for again, and paying it back to the
/// program the executor can spend leaves nothing stranded. Their
/// commitments still differ, which is the whole point of the pair.
///
/// The reserve arrives in the target's own printed spelling from the
/// executor's report and is carried here in the order the target commits
/// to it in, which is the reverse of the order it prints it in.
#[must_use]
pub fn sponsor_reserve_manifest(
    reserve_asset: [u8; 32],
    sponsor_program: &[u8],
) -> ConfidentialFixtureManifest {
    ConfidentialFixtureManifest {
        handle: sponsor_reserve_handle(),
        material_class: PublicDisposableTestMaterial::EXPECTED,
        derivation_profile: FixtureDerivationProfile::GuideCtfV1,
        profiles: selected_profiles(),
        retry_limit: MAX_PARITY_COUNTER,
        explicit_asset: reserve_asset,
        // The funding input is the executor's explicit change coin, so
        // it contributes a zero value blinder and the two output
        // blinders come out ordered additive inverses. The coin that
        // gets SPENT is the derived one; a single output would have had
        // to carry the solved blinder, and for a zero input sum that
        // blinder is zero and hides nothing.
        input_blinder_sum: [0_u8; 32],
        outputs: vec![
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Primary,
                semantic_amount: SPONSOR_RESERVE_AMOUNTS[0],
                output_program: sponsor_program.to_vec(),
            },
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Balancing,
                semantic_amount: SPONSOR_RESERVE_AMOUNTS[1],
                output_program: sponsor_program.to_vec(),
            },
        ],
    }
}

/// Registers the sponsor reserve case and freezes the registry.
///
/// # Errors
///
/// [`RegistrationRefusal`] where the manifest is not one the registry
/// admits.
pub fn frozen_sponsor_reserve_registry(
    reserve_asset: [u8; 32],
    sponsor_program: &[u8],
) -> Result<FrozenConfidentialFixtureRegistry, RegistrationRefusal> {
    let mut registry = ConfidentialFixtureRegistry::new();
    registry.register(sponsor_reserve_manifest(reserve_asset, sponsor_program))?;
    Ok(registry.freeze())
}

/// The subject of the confidential sponsor-funding step, against one
/// frozen registry.
///
/// # Why the destinations are stated when the manifest already holds
/// them
///
/// They are the same programs, and they are stated twice on purpose. The
/// manifest's copy is inside the digest and the request's copy is what
/// the executor builds outputs from, so a request whose programs drifted
/// from the ones its digest was taken over resolves to a different
/// digest and refuses. Deriving the request's copy FROM the manifest
/// would remove that check by making the two the same value.
#[must_use]
pub fn sponsor_reserve_subject(
    digest: ConfidentialFixtureDigest,
    sponsor_program: &[u8],
) -> TargetConfidentialSponsorFundingSubject {
    TargetConfidentialSponsorFundingSubject {
        destinations: (0..SPONSOR_RESERVE_OUTPUTS)
            .map(|_| ConfidentialFundingDestination {
                output_program: sponsor_program.to_vec(),
            })
            .collect(),
        binding: ConfidentialFundingBinding {
            fixture_handle: sponsor_reserve_handle(),
            fixture_digest: digest,
            profiles: selected_profiles(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SPONSOR_RESERVE_AMOUNTS, SPONSOR_RESERVE_OUTPUTS, frozen_sponsor_reserve_registry,
        sponsor_reserve_manifest,
    };
    use target_elements_conformance::confidential_fixture::{
        FixtureOutputRole, sponsor_reserve_handle,
    };

    /// A program of the class the executor pays sponsor coins to:
    /// version zero over a twenty-byte payload.
    const PROGRAM: [u8; 22] = [
        0x00, 0x14, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD,
        0xEE, 0xFF, 0x01, 0x02, 0x03, 0x04, 0x05,
    ];

    /// A reserve identifier standing in for the chain's.
    const RESERVE: [u8; 32] = [0x5A; 32];

    /// The registry admits the case, and holds a digest for it.
    ///
    /// The registration clauses this manifest has to satisfy are not
    /// obvious from reading it — exactly one output solves the balance,
    /// every amount is positive and semantic, every non-fee output
    /// states a program — so the admission is run rather than assumed.
    #[test]
    fn the_registry_admits_the_sponsor_reserve_case() {
        let registry = frozen_sponsor_reserve_registry(RESERVE, &PROGRAM)
            .expect("the registry refused the sponsor reserve manifest");
        assert!(
            registry
                .registered_digest(&sponsor_reserve_handle())
                .is_some()
        );
    }

    /// The two coins carry the same amount and are told apart by their
    /// commitments alone.
    ///
    /// This is the property the equal amounts exist for, and it is
    /// checked against the registry's own derived openings rather than
    /// argued: a run that had quietly given the two outputs different
    /// amounts would pass a weaker claim here.
    #[test]
    fn the_two_coins_share_an_amount_and_not_a_commitment() {
        assert_eq!(SPONSOR_RESERVE_AMOUNTS[0], SPONSOR_RESERVE_AMOUNTS[1]);
        let registry = frozen_sponsor_reserve_registry(RESERVE, &PROGRAM)
            .expect("the registry refused the sponsor reserve manifest");
        let resolved = registry
            .resolve(
                &sponsor_reserve_handle(),
                registry
                    .registered_digest(&sponsor_reserve_handle())
                    .expect("the registry holds no digest for a case it registered"),
            )
            .expect("the registry did not resolve a case it registered");
        let commitments = resolved
            .value_commitments()
            .expect("the resolved case carries no commitments");
        assert_eq!(commitments.len(), SPONSOR_RESERVE_OUTPUTS);
        assert_ne!(
            commitments[0], commitments[1],
            "two equal amounts committed under different blinders produced one point",
        );
    }

    /// The spendable coin's blinder is DERIVED and the other's is
    /// solved.
    ///
    /// The order is load-bearing rather than cosmetic. For a zero input
    /// blinder sum the solved blinder of a single-output case is zero,
    /// and a commitment under a zero blinder hides nothing; putting the
    /// spent coin in the solved position of a two-output case would not
    /// be that degeneracy, but it would make the coin the run depends on
    /// the one whose blinder no derivation states.
    #[test]
    fn the_spent_coin_is_the_derived_one() {
        let manifest = sponsor_reserve_manifest(RESERVE, &PROGRAM);
        assert_eq!(manifest.outputs[0].role, FixtureOutputRole::Primary);
        assert_eq!(manifest.outputs[1].role, FixtureOutputRole::Balancing);
        assert!(!manifest.outputs[0].role.solves_the_balance());
        assert!(manifest.outputs[1].role.solves_the_balance());
    }
}
