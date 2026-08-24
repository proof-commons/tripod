//! The confidential predecessor ceremony: what is registered, what is
//! asked for, and what a validated record is assembled from.
//!
//! # The ceremony plan selects the contract, and the executor never does
//!
//! The reproducibility contract is a per-ceremony typed selection made
//! here, in the evidence lane that will have to live with what its own
//! report claims. The executor advertises which contracts it supports
//! and an unsupported selection is refused before construction;
//! advertisement constrains and never chooses.
//!
//! # Why the asset is issued by a separate explicit step
//!
//! The fixture binds the explicit protocol asset, and the digest binds
//! the fixture. An issuing confidential step would therefore have to
//! bind a value that does not exist until the step has already run. So
//! the ceremony issues the disposable asset through the existing
//! explicit arm, registers the fixture against the asset that step
//! reported, and then asks the confidential arm for a non-issuing step
//! against it.
//!
//! That ordering is also what keeps the registry's authority intact: the
//! handle and the digest the request carries are the registry's own, and
//! the executor resolves them in its own catalogue rather than being
//! told what they mean.
//!
//! # Nothing here is an expectation about a chain
//!
//! The plan states what to ask for. What came back is read from the
//! transcript and from the mined bytes, by the validator in the
//! conformance package, against the first-party commitment oracle. This
//! module never compares a value with itself.

use target_elements::ReproducibilityContract;
use target_elements_conformance::confidential_fixture::{
    ConfidentialFixtureManifest, ConfidentialFixtureOutput, ConfidentialFixtureRegistry,
    FixtureDerivationProfile, FixtureOutputRole, FrozenConfidentialFixtureRegistry,
    MAX_PARITY_COUNTER, PublicDisposableTestMaterial, RegistrationRefusal, predecessor_handle,
};
use target_elements_conformance::confidential_funding::MinedInclusionOracle;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    ConfidentialFundingBinding, ConfidentialFundingDestination, ConfidentialFundingProfiles,
    FundingCustodyProfile, FundingMaterializerProfile, FundingRepresentationProfile,
    NativeOperationResponse, OperationCaseId, OperationStepKind, OperationSubject,
    TargetConfidentialFundingSubject, TargetFundingSubject,
};

/// The caller's own name for the step that issues the protocol asset.
pub const ISSUE_STEP: &str = "issue-confidential-protocol-asset";

/// The caller's own name for the confidential funding step.
pub const FUND_STEP: &str = "fund-confidential-predecessor";

/// The two programs the predecessor's outputs pay, in fixed order.
///
/// Distinct on purpose. A rangeproof binds the program it was built for,
/// so two outputs paying one program would leave the census member that
/// covers binding-to-THIS-output unable to fail.
pub const PREDECESSOR_PROGRAMS: [&[u8]; 2] = [&[0x51], &[0x51, 0x75, 0x51]];

/// The semantic amounts the predecessor's outputs carry, in fixed order.
///
/// Public disposable test material on a chain this run creates and
/// destroys. They are stated here and in the executor's own catalogue,
/// and the fixture digest is what detects the two drifting apart.
pub const PREDECESSOR_AMOUNTS: [u64; 2] = [700_000_000, 300_000_000];

/// What the issuing step creates, so that the reserve the confidential
/// step draws on exists at all.
const ISSUE_OUTPUTS: u8 = 1;

/// What the issuing step pays into each of those outputs.
const ISSUE_AMOUNT_PER_OUTPUT: u64 = 1;

/// The program the issuing step pays into.
const ISSUE_PROGRAM: [u8; 1] = [0x51];

/// The profiles this ceremony selects.
///
/// Byte identity is the reference contract, and it is what the
/// deterministic central public fixtures exist to serve. The selection
/// is made here rather than inferred anywhere.
#[must_use]
pub const fn selected_profiles() -> ConfidentialFundingProfiles {
    ConfidentialFundingProfiles {
        representation: FundingRepresentationProfile::ExplicitAssetConfidentialValue,
        custody: FundingCustodyProfile::CentralPublicFixtures,
        materializer: FundingMaterializerProfile::GuideCtfDeterministicV1,
        reproducibility_contract: ReproducibilityContract::ByteIdentity,
    }
}

/// The predecessor's manifest, against one issued protocol asset.
///
/// The asset arrives in the target's own printed spelling and is carried
/// into the manifest in the order the target commits to it in, which is
/// the reverse of the order it prints it in.
#[must_use]
pub fn predecessor_manifest(explicit_asset: [u8; 32]) -> ConfidentialFixtureManifest {
    ConfidentialFixtureManifest {
        handle: predecessor_handle(),
        material_class: PublicDisposableTestMaterial::EXPECTED,
        derivation_profile: FixtureDerivationProfile::GuideCtfV1,
        profiles: selected_profiles(),
        retry_limit: MAX_PARITY_COUNTER,
        explicit_asset,
        // The one funding input is explicit, so it contributes a zero
        // value blinder and the two output blinders come out ordered
        // additive inverses.
        input_blinder_sum: [0_u8; 32],
        outputs: vec![
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Primary,
                semantic_amount: PREDECESSOR_AMOUNTS[0],
                output_program: PREDECESSOR_PROGRAMS[0].to_vec(),
            },
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Balancing,
                semantic_amount: PREDECESSOR_AMOUNTS[1],
                output_program: PREDECESSOR_PROGRAMS[1].to_vec(),
            },
        ],
    }
}

/// Registers the predecessor and freezes the registry.
///
/// # Errors
///
/// [`RegistrationRefusal`] where the manifest is not one the registry
/// admits.
pub fn frozen_predecessor_registry(
    explicit_asset: [u8; 32],
) -> Result<FrozenConfidentialFixtureRegistry, RegistrationRefusal> {
    let mut registry = ConfidentialFixtureRegistry::new();
    registry.register(predecessor_manifest(explicit_asset))?;
    Ok(registry.freeze())
}

/// One printed identifier, back in the order the target commits to it.
#[must_use]
pub fn serialized_identifier(printed: &str) -> Option<[u8; 32]> {
    if printed.len() != 64 {
        return None;
    }
    let mut bytes = [0_u8; 32];
    for (index, slot) in bytes.iter_mut().enumerate() {
        let pair = printed.get(index * 2..index * 2 + 2)?;
        *slot = u8::from_str_radix(pair, 16).ok()?;
    }
    bytes.reverse();
    Some(bytes)
}

/// Why the ceremony could not state its next step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PredecessorPlanRefusal {
    /// The issuing step reported no asset.
    IssuedAssetAbsent,
    /// The reported asset is not an identifier.
    IssuedAssetMalformed,
    /// The registry refused the manifest.
    Registration {
        /// Its own typed cause.
        refusal: RegistrationRefusal,
    },
    /// The registry registered the case and then did not hold its
    /// digest, which is a defect in the registry rather than in the run.
    RegisteredDigestAbsent,
}

/// The two-step ceremony that funds one confidential predecessor.
///
/// It states what to ask for and keeps what it registered. It decides
/// nothing about what came back: the record is assembled by the
/// validator, against the first-party oracle and the mined bytes.
#[derive(Debug, Default)]
pub struct ConfidentialPredecessorPlan {
    registry: Option<FrozenConfidentialFixtureRegistry>,
    subject: Option<TargetConfidentialFundingSubject>,
    issued_asset: Option<String>,
    refusal: Option<PredecessorPlanRefusal>,
    asked: usize,
}

impl ConfidentialPredecessorPlan {
    /// One plan, before its first step.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The frozen registry, once the asset is known.
    #[must_use]
    pub const fn registry(&self) -> Option<&FrozenConfidentialFixtureRegistry> {
        self.registry.as_ref()
    }

    /// The exact confidential subject that was sent.
    #[must_use]
    pub const fn subject(&self) -> Option<&TargetConfidentialFundingSubject> {
        self.subject.as_ref()
    }

    /// The asset the issuing step reported.
    #[must_use]
    pub const fn issued_asset(&self) -> Option<&String> {
        self.issued_asset.as_ref()
    }

    /// Why the plan stopped, where it did.
    #[must_use]
    pub const fn refusal(&self) -> Option<PredecessorPlanRefusal> {
        self.refusal
    }

    /// The identity the confidential step is asked under.
    ///
    /// Named once so the run that sends it and the reader that looks it
    /// up in the transcript cannot spell it two ways.
    #[must_use]
    pub fn funding_case() -> OperationCaseId {
        OperationCaseId {
            operation: OperationStepKind::FundConfidential,
            step: FUND_STEP.to_owned(),
        }
    }

    /// The confidential subject, built against one reported asset.
    fn confidential_subject(
        &mut self,
        printed: &str,
    ) -> Result<TargetConfidentialFundingSubject, PredecessorPlanRefusal> {
        let asset =
            serialized_identifier(printed).ok_or(PredecessorPlanRefusal::IssuedAssetMalformed)?;
        let registry = frozen_predecessor_registry(asset)
            .map_err(|refusal| PredecessorPlanRefusal::Registration { refusal })?;
        let handle = predecessor_handle();
        let digest = *registry
            .registered_digest(&handle)
            .ok_or(PredecessorPlanRefusal::RegisteredDigestAbsent)?;
        self.registry = Some(registry);
        Ok(TargetConfidentialFundingSubject {
            issue_asset: false,
            asset: Some(printed.to_owned()),
            destinations: PREDECESSOR_PROGRAMS
                .iter()
                .map(|program| ConfidentialFundingDestination {
                    output_program: (*program).to_vec(),
                })
                .collect(),
            binding: ConfidentialFundingBinding {
                fixture_handle: handle,
                fixture_digest: digest,
                profiles: selected_profiles(),
            },
        })
    }
}

impl TargetOperationPlanner for ConfidentialPredecessorPlan {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        match previous {
            None => {
                self.asked += 1;
                Ok(Some(OperationStep::new(
                    ISSUE_STEP,
                    OperationSubject::Funding(Box::new(TargetFundingSubject {
                        issue_asset: true,
                        asset: None,
                        output_program: ISSUE_PROGRAM.to_vec(),
                        outputs: ISSUE_OUTPUTS,
                        amount_per_output: ISSUE_AMOUNT_PER_OUTPUT,
                    })),
                )))
            }
            Some((_, response)) if self.asked == 1 => {
                let printed = response.issued_asset.clone().ok_or_else(|| {
                    self.refusal = Some(PredecessorPlanRefusal::IssuedAssetAbsent);
                    PlanRefused
                })?;
                let subject = self.confidential_subject(&printed).map_err(|refusal| {
                    self.refusal = Some(refusal);
                    PlanRefused
                })?;
                self.issued_asset = Some(printed);
                self.subject = Some(subject.clone());
                self.asked += 1;
                Ok(Some(OperationStep::new(
                    FUND_STEP,
                    OperationSubject::ConfidentialFunding(Box::new(subject)),
                )))
            }
            Some(_) => Ok(None),
        }
    }
}

/// What the adapter reported about where the transaction was mined.
///
/// # This is the adapter's observation and not a second query
///
/// The inclusion condition asks whether the named block holds the named
/// transaction. Answering it independently would need a second reading
/// of the chain, and the funding wire carries no member a block's
/// contents could arrive in: the response states a block hash, a height,
/// and the raw transaction, and nothing else about the block.
///
/// So this oracle states exactly what it has — that the answer named a
/// block and a height, and that the identity it is asked about is the
/// one recomputed from the bytes that answer carried. The adapter checks
/// the block's own transaction list on its side before reporting, which
/// is a real check on the target's side of the boundary and is NOT a
/// second origin on this side. The distinction is recorded here rather
/// than smoothed over, because a reader entitled to think the harness
/// re-queried the chain would be reading more into the census than the
/// wire can carry.
#[derive(Clone, Debug, Default)]
pub struct AdapterReportedInclusion {
    transaction_id: String,
}

impl AdapterReportedInclusion {
    /// One oracle, bound to the identity recomputed from the mined bytes.
    #[must_use]
    pub const fn recomputed_from(transaction_id: String) -> Self {
        Self { transaction_id }
    }
}

impl MinedInclusionOracle for AdapterReportedInclusion {
    fn contains(&self, block_hash: &str, block_height: u32, transaction_id: &str) -> bool {
        !block_hash.is_empty() && block_height > 0 && transaction_id == self.transaction_id
    }
}
