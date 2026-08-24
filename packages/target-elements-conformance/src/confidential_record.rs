//! The predecessor slice's regions, its independent agreement census,
//! and the validated confidential funding record.
//!
//! # Every comparison is between two different origins
//!
//! The workspace already owns this rule and already paid for it. A
//! comparison of a value with itself, of a value with a helper both
//! sides call, or of a materialized value with a reference computed by
//! the same library the materializer used, is not evidence, and it is
//! refused at the boundary that assembles it rather than left to a
//! reviewer's attention. [`AgreementOrigin`] names the four origins, and
//! [`OutputAgreementCensus::assemble`] refuses a field whose two sides
//! carry the same one.
//!
//! The commitment comparison carries the property in the type system
//! instead: [`RecomputedCommitment`] can be built only by first-party
//! bignum arithmetic over published constants, [`ReadBackCommitment`]
//! only from bytes a target mined, and [`compare_commitments`] takes one
//! of each. There is no way to spell a comparison of a readback with a
//! readback.
//!
//! # A validated record is not an offered record that passed
//!
//! [`validate_confidential_funding_record`] does not take a record and
//! check it. It takes the exchange, the registered fixture, and the
//! oracles, and BUILDS the record by recomputing every member in the
//! guide's fixed order. A raw wire record therefore cannot become a
//! validated record, and a caller-authored outcome cannot discharge one.
//!
//! # What this record does not say
//!
//! Funding evidence is one evidence role and never seven. The record
//! carries the exhaustive non-claims of [`CandidateFundingNonClaim`] and
//! the exhaustive exclusions of [`CanonicalFundingExclusion`], and it
//! reports no transfer, no authorization, no conservation, no
//! minimality, no production privacy, and no matrix discharge.

use std::collections::BTreeSet;

use target_elements::ReproducibilityContract;

use crate::commitment_oracle::commitment::{self, CommitmentDefect};
use crate::commitment_oracle::curve::{self, Group, PREFIXED_POINT_BYTES};
use crate::confidential_fixture::{
    COMMITMENT_BYTES, ConfidentialFixtureOutput, FixtureOpenings, FixtureOutputRole,
    ResolvedFixture,
};
use crate::confidential_funding::{
    ConfidentialFundingExchange, ConfidentialFundingObservation, ConfidentialFundingOracles,
    ConfidentialFundingRefusal, DecodedAssetField, DecodedFundingOutput, DecodedFundingTransaction,
    DecodedValueField, RangeproofVerifier, ReadbackDerivedFunding, ReadbackDerivedOutput,
    bind_confidential_funding,
};
use crate::protocol::{
    ConfidentialFixtureDigest, ConfidentialFixtureHandle, ConfidentialFundedOutput,
    ConfidentialFundingProfiles, ExecutorHandshake, NativeOperationResponse, ObservedOutcomeLayer,
    TargetConfidentialFundingSubject, WireOutpoint,
};
use crate::report::ObservedEnvironment;

/// Which of the four origins one side of a comparison came from.
///
/// The vocabulary is the workspace's own classification, carried here
/// unchanged rather than restated. A comparison requires two different
/// members of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AgreementOrigin {
    /// The construction's own output, never its own expectation.
    Materialized,
    /// First-party arithmetic over published constants; independence
    /// unqualified.
    Recomputed,
    /// The reference implementation the target itself vendors;
    /// conformance evidence and never independence.
    Reference,
    /// The target's observation, decoded from mined bytes.
    ReadBack,
}

impl std::fmt::Display for AgreementOrigin {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Materialized => "materialized",
            Self::Recomputed => "recomputed",
            Self::Reference => "reference",
            Self::ReadBack => "read back",
        };
        formatter.write_str(text)
    }
}

/// One commitment computed by first-party bignum arithmetic.
///
/// The only constructor recomputes it from an asset, an amount, and a
/// blinder through the first-party oracle, so a value of this type
/// cannot have come from anywhere else. That is the independence claim
/// carried by the type rather than by a comment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecomputedCommitment([u8; COMMITMENT_BYTES]);

impl RecomputedCommitment {
    /// Recomputes one commitment from an opening.
    ///
    /// # Errors
    ///
    /// [`CommitmentDefect`] where the asset, the scalar, or the result
    /// is refused.
    pub fn recompute(
        explicit_asset: &[u8; 32],
        semantic_amount: u64,
        value_blinder: &[u8],
    ) -> Result<Self, CommitmentDefect> {
        commitment::commitment(explicit_asset, semantic_amount, value_blinder).map(Self)
    }

    /// The serialized commitment.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; COMMITMENT_BYTES] {
        &self.0
    }

    /// The commitment's parity prefix.
    #[must_use]
    pub const fn prefix(&self) -> u8 {
        self.0[0]
    }
}

/// One commitment read out of bytes a target mined.
///
/// The only constructor reads it from a readback-derived output, so a
/// value of this type is the target's observation and never anyone's
/// expectation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadBackCommitment([u8; COMMITMENT_BYTES]);

impl ReadBackCommitment {
    /// Reads one commitment out of a readback-derived output.
    ///
    /// # Errors
    ///
    /// [`ReadBackWidthRefused`] where the decoded field is not a
    /// committed field's width.
    pub fn from_readback(output: &ReadbackDerivedOutput) -> Result<Self, ReadBackWidthRefused> {
        let bytes: [u8; COMMITMENT_BYTES] = output
            .value_commitment
            .as_slice()
            .try_into()
            .map_err(|_| ReadBackWidthRefused)?;
        Ok(Self(bytes))
    }

    /// The serialized commitment.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; COMMITMENT_BYTES] {
        &self.0
    }

    /// The commitment's parity prefix.
    #[must_use]
    pub const fn prefix(&self) -> u8 {
        self.0[0]
    }
}

/// A decoded commitment field that is not a committed field's width.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadBackWidthRefused;

/// Who performed one independent commitment comparison.
///
/// One member, and it is the first-party bignum oracle. The reference
/// bindings may implement the proof materializer and may NOT implement
/// this, because they bind the same library the target vendors and their
/// agreement would be conformance evidence rather than independence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CommitmentChecker {
    /// The first-party bignum commitment oracle.
    FirstPartyBignumOracle,
}

impl std::fmt::Display for CommitmentChecker {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FirstPartyBignumOracle => {
                formatter.write_str("the first-party bignum commitment oracle")
            }
        }
    }
}

/// One independent commitment comparison and its verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommitmentCheck {
    checker: CommitmentChecker,
    recomputed: RecomputedCommitment,
    observed: ReadBackCommitment,
    agrees: bool,
}

impl CommitmentCheck {
    /// Who checked.
    #[must_use]
    pub const fn checker(&self) -> CommitmentChecker {
        self.checker
    }

    /// The first-party recomputation.
    #[must_use]
    pub const fn recomputed(&self) -> &RecomputedCommitment {
        &self.recomputed
    }

    /// The target's observation.
    #[must_use]
    pub const fn observed(&self) -> &ReadBackCommitment {
        &self.observed
    }

    /// Whether the two agreed.
    #[must_use]
    pub const fn agrees(&self) -> bool {
        self.agrees
    }
}

/// Compares a first-party recomputation with a target observation.
///
/// The signature is the guarantee: two different types, so a
/// recomputation cannot be compared with a recomputation and a readback
/// cannot be compared with a readback.
#[must_use]
pub fn compare_commitments(
    recomputed: RecomputedCommitment,
    observed: ReadBackCommitment,
) -> CommitmentCheck {
    CommitmentCheck {
        checker: CommitmentChecker::FirstPartyBignumOracle,
        recomputed,
        observed,
        agrees: recomputed.bytes() == observed.bytes(),
    }
}

/// Which region one transaction member belongs to.
///
/// Classification is exhaustive: there is no member the classifier may
/// decline to place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FundingRegion {
    /// One of the protocol outputs, at its fixture position.
    Protocol {
        /// Its position in fixture order.
        position: usize,
    },
    /// A member outside both balance equations.
    NonProtocol(NonProtocolFundingRegion),
}

/// The members that sit outside both two-output balance equations.
///
/// The region exists so that a fee can be paid without the fee becoming
/// an argument about whether the two-output equation closed. A
/// transaction whose balance depends on which region a member was put in
/// has been classified wrongly, not balanced cleverly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NonProtocolFundingRegion {
    /// The policy-asset fee, which the target writes with no program.
    PolicyFee,
    /// Policy-asset change returning to the funding party.
    PolicyChange,
}

impl std::fmt::Display for NonProtocolFundingRegion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::PolicyFee => "the policy-asset fee",
            Self::PolicyChange => "policy-asset change",
        };
        formatter.write_str(text)
    }
}

/// Why one member could not be classified.
///
/// Closed, with no catch-all. Every variant is a construction refusal
/// and none is a target verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RegionClassificationRefusal {
    /// The transaction carries fewer members than the protocol region
    /// needs.
    ProtocolCountShort {
        /// How many the fixture fixes.
        required: usize,
        /// How many the transaction carries.
        observed: usize,
    },
    /// A protocol position does not carry the protocol asset.
    ProtocolAssetAbsent {
        /// Which position.
        position: usize,
    },
    /// A protocol position does not carry a committed value.
    ProtocolValueNotCommitted {
        /// Which position.
        position: usize,
    },
    /// A member outside the protocol region carries the protocol asset.
    ProtocolAssetOutsideRegion {
        /// Which member.
        index: usize,
    },
    /// A member outside the protocol region carries a proof, which would
    /// change the witness census the protocol region owns.
    NonProtocolMemberCarriesProof {
        /// Which member.
        index: usize,
    },
    /// A member outside the protocol region carries a commitment, which
    /// would put it inside a balance equation it is not part of.
    NonProtocolMemberNotExplicit {
        /// Which member.
        index: usize,
    },
}

impl std::fmt::Display for RegionClassificationRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProtocolCountShort { required, observed } => write!(
                formatter,
                "the fixture fixes {required} protocol members and the transaction carries {observed}",
            ),
            Self::ProtocolAssetAbsent { position } => write!(
                formatter,
                "protocol position {position} does not carry the protocol asset",
            ),
            Self::ProtocolValueNotCommitted { position } => write!(
                formatter,
                "protocol position {position} does not carry a committed value",
            ),
            Self::ProtocolAssetOutsideRegion { index } => write!(
                formatter,
                "member {index} carries the protocol asset outside the protocol region",
            ),
            Self::NonProtocolMemberCarriesProof { index } => {
                write!(
                    formatter,
                    "member {index} carries a proof outside the protocol region"
                )
            }
            Self::NonProtocolMemberNotExplicit { index } => {
                write!(
                    formatter,
                    "member {index} carries a commitment outside the protocol region"
                )
            }
        }
    }
}

/// Places every member of one decoded transaction in a region.
///
/// Exhaustive by construction: the returned vector has one entry per
/// decoded output, in the transaction's own order, and there is no
/// member the classifier may decline to place.
///
/// # Errors
///
/// [`RegionClassificationRefusal`] where a member is unclassifiable, or
/// carries the protocol asset, a commitment, or a proof outside the
/// protocol region.
pub fn classify_funding_members(
    decoded: &DecodedFundingTransaction,
    protocol_asset: &str,
    protocol_members: usize,
) -> Result<Vec<FundingRegion>, RegionClassificationRefusal> {
    if decoded.outputs.len() < protocol_members {
        return Err(RegionClassificationRefusal::ProtocolCountShort {
            required: protocol_members,
            observed: decoded.outputs.len(),
        });
    }
    let mut regions = Vec::with_capacity(decoded.outputs.len());
    for (index, output) in decoded.outputs.iter().enumerate() {
        if index < protocol_members {
            match &output.asset {
                DecodedAssetField::Explicit(asset) if asset == protocol_asset => {}
                DecodedAssetField::Explicit(_) | DecodedAssetField::Commitment(_) => {
                    return Err(RegionClassificationRefusal::ProtocolAssetAbsent {
                        position: index,
                    });
                }
            }
            if !matches!(output.value, DecodedValueField::Commitment(_)) {
                return Err(RegionClassificationRefusal::ProtocolValueNotCommitted {
                    position: index,
                });
            }
            regions.push(FundingRegion::Protocol { position: index });
            continue;
        }
        if let DecodedAssetField::Explicit(asset) = &output.asset {
            if asset == protocol_asset {
                return Err(RegionClassificationRefusal::ProtocolAssetOutsideRegion { index });
            }
        }
        if !matches!(output.value, DecodedValueField::Explicit(_)) {
            return Err(RegionClassificationRefusal::NonProtocolMemberNotExplicit { index });
        }
        if !output.rangeproof.is_empty() || !output.surjection_proof.is_empty() {
            return Err(RegionClassificationRefusal::NonProtocolMemberCarriesProof { index });
        }
        let region = if output.program.is_empty() {
            NonProtocolFundingRegion::PolicyFee
        } else {
            NonProtocolFundingRegion::PolicyChange
        };
        regions.push(FundingRegion::NonProtocol(region));
    }
    Ok(regions)
}

/// The fixed census recomputation and raw readback must agree on.
///
/// Eight members and not one summary word, so that a report naming
/// agreement names which agreement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FundingAgreementField {
    /// The explicit protocol asset the output carries.
    Asset,
    /// The serialized value commitment.
    Commitment,
    /// The commitment's parity prefix.
    Parity,
    /// The nonce field.
    Nonce,
    /// The output program.
    Program,
    /// Presence, verification, empty surjection, and binding to THIS
    /// output.
    ProofShape,
    /// The coin the output creates.
    Outpoint,
    /// The mined index bound to the witness-bearing bytes.
    WitnessTransactionIdentity,
}

impl FundingAgreementField {
    /// Every member, in the order the census states them.
    pub const ALL: [Self; 8] = [
        Self::Asset,
        Self::Commitment,
        Self::Parity,
        Self::Nonce,
        Self::Program,
        Self::ProofShape,
        Self::Outpoint,
        Self::WitnessTransactionIdentity,
    ];
}

impl std::fmt::Display for FundingAgreementField {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Asset => "asset",
            Self::Commitment => "commitment",
            Self::Parity => "parity",
            Self::Nonce => "nonce",
            Self::Program => "program",
            Self::ProofShape => "proof shape",
            Self::Outpoint => "outpoint",
            Self::WitnessTransactionIdentity => "witness transaction identity",
        };
        formatter.write_str(text)
    }
}

/// One field's agreement, with the two origins it was taken between.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldAgreement {
    field: FundingAgreementField,
    expectation: AgreementOrigin,
    observation: AgreementOrigin,
    agrees: bool,
}

impl FieldAgreement {
    /// Which field.
    #[must_use]
    pub const fn field(&self) -> FundingAgreementField {
        self.field
    }

    /// Where the expectation came from.
    #[must_use]
    pub const fn expectation(&self) -> AgreementOrigin {
        self.expectation
    }

    /// Where the observation came from.
    #[must_use]
    pub const fn observation(&self) -> AgreementOrigin {
        self.observation
    }

    /// Whether the two agreed.
    #[must_use]
    pub const fn agrees(&self) -> bool {
        self.agrees
    }
}

/// Why an agreement census could not be assembled or did not hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AgreementRefusal {
    /// One field's two sides carry the same origin.
    ///
    /// A comparison of a value with itself is not evidence, and it is
    /// refused where it is assembled rather than reported as agreement.
    SelfAttestation {
        /// Which field.
        field: FundingAgreementField,
    },
    /// One field disagreed.
    FieldDisagrees {
        /// Which output.
        output: usize,
        /// Which field.
        field: FundingAgreementField,
    },
    /// The census does not carry every member.
    CensusIncomplete {
        /// Which output.
        output: usize,
    },
}

impl std::fmt::Display for AgreementRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SelfAttestation { field } => {
                write!(
                    formatter,
                    "the {field} comparison has one origin on both sides"
                )
            }
            Self::FieldDisagrees { output, field } => {
                write!(formatter, "the {field} of output {output} disagrees")
            }
            Self::CensusIncomplete { output } => {
                write!(formatter, "the census of output {output} is incomplete")
            }
        }
    }
}

/// One output's complete agreement census.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputAgreementCensus {
    output: usize,
    fields: Vec<FieldAgreement>,
}

impl OutputAgreementCensus {
    /// Assembles one census, refusing a comparison of a value with
    /// itself.
    ///
    /// # Errors
    ///
    /// [`AgreementRefusal::SelfAttestation`] where a field's two sides
    /// carry one origin, and [`AgreementRefusal::CensusIncomplete`]
    /// where a member of the fixed census is missing.
    pub fn assemble(output: usize, fields: Vec<FieldAgreement>) -> Result<Self, AgreementRefusal> {
        for entry in &fields {
            if entry.expectation == entry.observation {
                return Err(AgreementRefusal::SelfAttestation { field: entry.field });
            }
        }
        let present: BTreeSet<FundingAgreementField> =
            fields.iter().map(FieldAgreement::field).collect();
        let required: BTreeSet<FundingAgreementField> =
            FundingAgreementField::ALL.into_iter().collect();
        if present != required {
            return Err(AgreementRefusal::CensusIncomplete { output });
        }
        Ok(Self { output, fields })
    }

    /// Which output.
    #[must_use]
    pub const fn output(&self) -> usize {
        self.output
    }

    /// Every field's agreement.
    #[must_use]
    pub fn fields(&self) -> &[FieldAgreement] {
        &self.fields
    }

    /// The first field that disagreed, where one did.
    ///
    /// # Errors
    ///
    /// [`AgreementRefusal::FieldDisagrees`] naming that field.
    pub fn require_agreement(&self) -> Result<(), AgreementRefusal> {
        for entry in &self.fields {
            if !entry.agrees {
                return Err(AgreementRefusal::FieldDisagrees {
                    output: self.output,
                    field: entry.field,
                });
            }
        }
        Ok(())
    }
}

/// The four form facts one protocol output must carry.
///
/// A census rather than a summary word: a record naming three of them is
/// the thing this vocabulary exists to make unsayable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FundingOutputForm {
    /// The value is a commitment.
    ConfidentialCommitment,
    /// The asset is the explicit protocol asset.
    ExplicitProtocolAsset,
    /// The rangeproof is present and verifies against this output.
    ValidRangeproof,
    /// The surjection-proof field is empty.
    EmptySurjectionProof,
}

impl FundingOutputForm {
    /// Every fact, in the order the census states them.
    pub const ALL: [Self; 4] = [
        Self::ConfidentialCommitment,
        Self::ExplicitProtocolAsset,
        Self::ValidRangeproof,
        Self::EmptySurjectionProof,
    ];

    /// The complete census.
    #[must_use]
    pub fn complete() -> BTreeSet<Self> {
        Self::ALL.into_iter().collect()
    }
}

/// What a canonical funding record never carries.
///
/// Exhaustive. Public fixture data lives in its chartered attachment; it
/// is not smuggled into a schema that excludes it, and the schema is not
/// loosened so that it fits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CanonicalFundingExclusion {
    /// Private amounts.
    PrivateAmounts,
    /// Fixture openings.
    FixtureOpenings,
    /// Value blinders.
    ValueBlinders,
    /// Nonce inputs.
    NonceInputs,
    /// Proof inputs.
    ProofInputs,
    /// Wallet data.
    WalletData,
    /// Credentials.
    Credentials,
    /// Environment values.
    EnvironmentValues,
    /// Diagnostics.
    Diagnostics,
}

impl CanonicalFundingExclusion {
    /// Every exclusion, in the order the vocabulary states them.
    pub const ALL: [Self; 9] = [
        Self::PrivateAmounts,
        Self::FixtureOpenings,
        Self::ValueBlinders,
        Self::NonceInputs,
        Self::ProofInputs,
        Self::WalletData,
        Self::Credentials,
        Self::EnvironmentValues,
        Self::Diagnostics,
    ];
}

/// What a candidate funding result does not establish.
///
/// Exhaustive, carried by every validated record and every report, and
/// checked rather than written down.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CandidateFundingNonClaim {
    /// Production opening, key, nonce, seed, blinder, wallet, or
    /// credential custody.
    ProductionCustodyNotEstablished,
    /// Production-quality randomness, blinding, proof generation,
    /// signing, erasure, or side-channel resistance.
    ProductionCryptographyNotEstablished,
    /// A production multi-owner blinding or signing protocol.
    ProductionMultiOwnerProtocolNotEstablished,
    /// Owner anonymity, graph privacy, count privacy, timing privacy,
    /// wallet privacy, or universal transaction confidentiality.
    PrivacyNotEstablished,
    /// That public deterministic fixture openings are secret from an
    /// observer.
    PublicFixturesNotSecret,
    /// That the stock target RPC surface supports the selected hybrid
    /// representation.
    StockRpcHybridSupportNotEstablished,
    /// That funding evidence proves a live transfer, a safety relation,
    /// a disclosure-minimality relation, or a resource result.
    FundingDoesNotProveTransferSafetyMinimalityOrResource,
    /// That malformed private rejections substitute for an accepted
    /// positive private transaction.
    MalformedRejectionDoesNotReplaceControl,
    /// That the owner-sighash profile is implemented or reviewed here.
    OwnerSighashNotEstablishedHere,
    /// That a candidate interface is final, stable, production-capable,
    /// or released.
    CandidateInterfaceNotFinal,
}

impl CandidateFundingNonClaim {
    /// Every non-claim, in the order the vocabulary states them.
    pub const ALL: [Self; 10] = [
        Self::ProductionCustodyNotEstablished,
        Self::ProductionCryptographyNotEstablished,
        Self::ProductionMultiOwnerProtocolNotEstablished,
        Self::PrivacyNotEstablished,
        Self::PublicFixturesNotSecret,
        Self::StockRpcHybridSupportNotEstablished,
        Self::FundingDoesNotProveTransferSafetyMinimalityOrResource,
        Self::MalformedRejectionDoesNotReplaceControl,
        Self::OwnerSighashNotEstablishedHere,
        Self::CandidateInterfaceNotFinal,
    ];
}

/// One protocol output, as the validator recomputed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedFundingOutput {
    output_index: usize,
    role: FixtureOutputRole,
    outpoint: WireOutpoint,
    explicit_asset: String,
    value_commitment: Vec<u8>,
    commitment_prefix: u8,
    nonce: Vec<u8>,
    output_program: Vec<u8>,
    rangeproof: Vec<u8>,
    surjection_proof: Vec<u8>,
    witness_transaction_id: String,
    form: BTreeSet<FundingOutputForm>,
}

impl ValidatedFundingOutput {
    /// Its position in fixture order.
    #[must_use]
    pub const fn output_index(&self) -> usize {
        self.output_index
    }

    /// What the fixture marked it.
    #[must_use]
    pub const fn role(&self) -> FixtureOutputRole {
        self.role
    }

    /// The coin it creates.
    #[must_use]
    pub const fn outpoint(&self) -> &WireOutpoint {
        &self.outpoint
    }

    /// The explicit protocol asset it carries.
    #[must_use]
    pub fn explicit_asset(&self) -> &str {
        &self.explicit_asset
    }

    /// The serialized value commitment.
    #[must_use]
    pub fn value_commitment(&self) -> &[u8] {
        &self.value_commitment
    }

    /// The commitment's parity prefix.
    #[must_use]
    pub const fn commitment_prefix(&self) -> u8 {
        self.commitment_prefix
    }

    /// The nonce field.
    #[must_use]
    pub fn nonce(&self) -> &[u8] {
        &self.nonce
    }

    /// The program it pays.
    #[must_use]
    pub fn output_program(&self) -> &[u8] {
        &self.output_program
    }

    /// The rangeproof it carries.
    #[must_use]
    pub fn rangeproof(&self) -> &[u8] {
        &self.rangeproof
    }

    /// The surjection-proof field, which the hybrid form leaves empty.
    #[must_use]
    pub fn surjection_proof(&self) -> &[u8] {
        &self.surjection_proof
    }

    /// The witness-bearing identity this output is bound to.
    #[must_use]
    pub fn witness_transaction_id(&self) -> &str {
        &self.witness_transaction_id
    }

    /// The complete form census.
    #[must_use]
    pub const fn form(&self) -> &BTreeSet<FundingOutputForm> {
        &self.form
    }
}

/// The summary, recomputed from the members rather than carried.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FundingSummary {
    protocol_outputs: usize,
    non_protocol_members: usize,
    agreeing_fields: usize,
    agreeing_commitments: usize,
    observed_parities: Vec<u8>,
    contract: ReproducibilityContract,
}

impl FundingSummary {
    /// How many protocol outputs the record binds.
    #[must_use]
    pub const fn protocol_outputs(&self) -> usize {
        self.protocol_outputs
    }

    /// How many members sat outside both balance equations.
    #[must_use]
    pub const fn non_protocol_members(&self) -> usize {
        self.non_protocol_members
    }

    /// How many census fields agreed, across every output.
    #[must_use]
    pub const fn agreeing_fields(&self) -> usize {
        self.agreeing_fields
    }

    /// How many independent commitment comparisons agreed.
    #[must_use]
    pub const fn agreeing_commitments(&self) -> usize {
        self.agreeing_commitments
    }

    /// The observed parity prefixes, in fixture order.
    #[must_use]
    pub fn observed_parities(&self) -> &[u8] {
        &self.observed_parities
    }

    /// The contract the run was judged under.
    #[must_use]
    pub const fn contract(&self) -> ReproducibilityContract {
        self.contract
    }
}

/// One confidential funding record, before validation gives it a name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfidentialFundingRecord {
    handshake: ExecutorHandshake,
    environment: ObservedEnvironment,
    request: TargetConfidentialFundingSubject,
    response: NativeOperationResponse,
    fixture_handle: ConfidentialFixtureHandle,
    fixture_digest: ConfidentialFixtureDigest,
    profiles: ConfidentialFundingProfiles,
    readback: ReadbackDerivedFunding,
    regions: Vec<FundingRegion>,
    outputs: Vec<ValidatedFundingOutput>,
    agreement: Vec<OutputAgreementCensus>,
    independent_commitments: Vec<CommitmentCheck>,
    observed_layer: ObservedOutcomeLayer,
    exclusions: BTreeSet<CanonicalFundingExclusion>,
    non_claims: BTreeSet<CandidateFundingNonClaim>,
    summary: FundingSummary,
}

impl ConfidentialFundingRecord {
    /// What the executor said it was.
    #[must_use]
    pub const fn handshake(&self) -> &ExecutorHandshake {
        &self.handshake
    }

    /// The deployment it ran against.
    #[must_use]
    pub const fn environment(&self) -> &ObservedEnvironment {
        &self.environment
    }

    /// The exact request, unredacted.
    #[must_use]
    pub const fn request(&self) -> &TargetConfidentialFundingSubject {
        &self.request
    }

    /// The exact response, unredacted.
    #[must_use]
    pub const fn response(&self) -> &NativeOperationResponse {
        &self.response
    }

    /// The registered case.
    #[must_use]
    pub const fn fixture_handle(&self) -> &ConfidentialFixtureHandle {
        &self.fixture_handle
    }

    /// The digest that case was registered under.
    #[must_use]
    pub const fn fixture_digest(&self) -> &ConfidentialFixtureDigest {
        &self.fixture_digest
    }

    /// The profiles the ceremony selected.
    #[must_use]
    pub const fn profiles(&self) -> &ConfidentialFundingProfiles {
        &self.profiles
    }

    /// What the chain holds, derived rather than echoed.
    #[must_use]
    pub const fn readback(&self) -> &ReadbackDerivedFunding {
        &self.readback
    }

    /// Every member's region.
    #[must_use]
    pub fn regions(&self) -> &[FundingRegion] {
        &self.regions
    }

    /// The protocol outputs, in fixture order.
    #[must_use]
    pub fn outputs(&self) -> &[ValidatedFundingOutput] {
        &self.outputs
    }

    /// The agreement census, one per protocol output.
    #[must_use]
    pub fn agreement(&self) -> &[OutputAgreementCensus] {
        &self.agreement
    }

    /// The independent commitment comparisons.
    #[must_use]
    pub fn independent_commitments(&self) -> &[CommitmentCheck] {
        &self.independent_commitments
    }

    /// The observed outcome layer.
    #[must_use]
    pub const fn observed_layer(&self) -> ObservedOutcomeLayer {
        self.observed_layer
    }

    /// What the record never carries.
    #[must_use]
    pub const fn exclusions(&self) -> &BTreeSet<CanonicalFundingExclusion> {
        &self.exclusions
    }

    /// What the record does not establish.
    #[must_use]
    pub const fn non_claims(&self) -> &BTreeSet<CandidateFundingNonClaim> {
        &self.non_claims
    }

    /// The recomputed summary.
    #[must_use]
    pub const fn summary(&self) -> &FundingSummary {
        &self.summary
    }
}

/// A confidential funding record that a validator built.
///
/// Its only constructor is [`validate_confidential_funding_record`]. A
/// raw wire record cannot become one, and a caller-authored outcome
/// cannot discharge one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedConfidentialFundingRecord {
    record: ConfidentialFundingRecord,
}

impl ValidatedConfidentialFundingRecord {
    /// The validated record.
    #[must_use]
    pub const fn record(&self) -> &ConfidentialFundingRecord {
        &self.record
    }

    /// Consumes the validated state, yielding the record.
    #[must_use]
    pub fn into_record(self) -> ConfidentialFundingRecord {
        self.record
    }
}

/// Which recomputation refused, so a failed record names the step it
/// failed.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FundingRecordRefusal {
    /// The request and response arms do not match, or a fallback
    /// occurred.
    Arm {
        /// The wire refusal that says so.
        refusal: ConfidentialFundingRefusal,
    },
    /// The handle, digest, profiles, request, response, and transaction
    /// do not all bind.
    Fixture {
        /// The wire refusal that says so.
        refusal: ConfidentialFundingRefusal,
    },
    /// The output order is not the fixture's fixed order.
    Order,
    /// A member is not exactly the hybrid tuple.
    Representation {
        /// Which output.
        output: usize,
    },
    /// The commitment prefixes are not the admitted pair.
    PrefixMask {
        /// Which output.
        output: usize,
    },
    /// The proof shape read from the readback does not hold.
    ProofShape {
        /// Which output.
        output: usize,
    },
    /// Submission, mining, readback, and outputs do not agree.
    Readback {
        /// The wire refusal that says so.
        refusal: ConfidentialFundingRefusal,
    },
    /// The summary does not recompute from the members.
    Summary,
    /// An independent commitment comparison did not agree.
    CommitmentCheck {
        /// Which output.
        output: usize,
    },
    /// The record would have carried an excluded value.
    CanonicalBoundary {
        /// Which exclusion.
        exclusion: CanonicalFundingExclusion,
    },
    /// A member could not be classified into a region.
    Region {
        /// The classification refusal.
        refusal: RegionClassificationRefusal,
    },
    /// The agreement census could not be assembled or did not hold.
    Agreement {
        /// The agreement refusal.
        refusal: AgreementRefusal,
    },
    /// A record claims a guarantee its contract does not carry.
    ReproducibilityClaimMismatch,
    /// The target reached a verdict and created nothing, so there is no
    /// funded observation to validate.
    NoFundedObservation,
}

impl std::fmt::Display for FundingRecordRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arm { refusal } => write!(formatter, "the arms do not match: {refusal}"),
            Self::Fixture { refusal } => write!(formatter, "the fixture does not bind: {refusal}"),
            Self::Order => formatter.write_str("the output order is not the fixture's"),
            Self::Representation { output } => {
                write!(formatter, "output {output} is not the hybrid tuple")
            }
            Self::PrefixMask { output } => {
                write!(formatter, "output {output} carries an unadmitted prefix")
            }
            Self::ProofShape { output } => {
                write!(
                    formatter,
                    "the proof shape of output {output} does not hold"
                )
            }
            Self::Readback { refusal } => write!(formatter, "the readback disagrees: {refusal}"),
            Self::Summary => formatter.write_str("the summary does not recompute"),
            Self::CommitmentCheck { output } => write!(
                formatter,
                "the independent commitment comparison of output {output} disagreed",
            ),
            Self::CanonicalBoundary { exclusion } => {
                write!(
                    formatter,
                    "the record would have carried an excluded value: {exclusion:?}"
                )
            }
            Self::Region { refusal } => write!(formatter, "a member is unclassified: {refusal}"),
            Self::Agreement { refusal } => write!(formatter, "{refusal}"),
            Self::ReproducibilityClaimMismatch => {
                formatter.write_str("the record claims a guarantee its contract does not carry")
            }
            Self::NoFundedObservation => {
                formatter.write_str("the target created nothing, so there is nothing to validate")
            }
        }
    }
}

/// Everything a validation needs, and nothing it could author.
pub struct ConfidentialFundingEvidence<'a> {
    /// What the executor said it was.
    pub handshake: &'a ExecutorHandshake,
    /// The deployment it ran against.
    pub environment: &'a ObservedEnvironment,
    /// The exact request that was sent.
    pub request: &'a TargetConfidentialFundingSubject,
    /// The exact response that came back.
    pub response: &'a NativeOperationResponse,
    /// The registered case the request names.
    pub fixture: &'a ResolvedFixture,
    /// The decoded mined transaction, whole, for region classification.
    pub decoded: &'a DecodedFundingTransaction,
    /// Whether a byte-identity rerun comparison was performed.
    ///
    /// Absent unless a second run compared bytes. A record under
    /// recorded randomness that carried one would be claiming the other
    /// contract's guarantee, which is
    /// [`FundingRecordRefusal::ReproducibilityClaimMismatch`].
    pub materialized_bytes_compared: bool,
}

/// Encodes one point as the reviewed encoding of a field writes it.
fn encode_field_point(
    point: &curve::Point,
    encoding: target_elements::ConfidentialFieldEncoding,
) -> Option<[u8; PREFIXED_POINT_BYTES]> {
    let (low, high) = encoding.committed_prefixes();
    let prefix = match encoding.parity() {
        target_elements::PointParityConvention::CompressedOddness => {
            if point.y().bit(0) {
                high
            } else {
                low
            }
        }
        target_elements::PointParityConvention::QuadraticResidue => {
            if point.has_square_y() {
                low
            } else {
                high
            }
        }
        // The convention that carries no prefix at all, and any
        // convention a later target contract adds. Neither can be
        // encoded from the pair this function was handed, so both
        // refuse rather than guess a prefix.
        target_elements::PointParityConvention::ImpliedEvenY => return None,
        _ => return None,
    };
    let mut encoded = [0_u8; PREFIXED_POINT_BYTES];
    encoded[0] = prefix;
    encoded[1..].copy_from_slice(&point.x_bytes());
    Some(encoded)
}

/// The nonce field one nonce input produces.
fn recomputed_nonce(nonce_input: &[u8; 32]) -> Option<[u8; PREFIXED_POINT_BYTES]> {
    let scalar = commitment::read_scalar(nonce_input).ok()?;
    let point = match curve::multiply_point(&scalar, &curve::base_point()) {
        Group::Affine(point) => point,
        Group::Identity => return None,
    };
    encode_field_point(
        &point,
        target_elements::reviewed_confidential_review_facts().nonce(),
    )
}

/// One field's agreement, stated with its two origins.
const fn agreement(
    field: FundingAgreementField,
    expectation: AgreementOrigin,
    observation: AgreementOrigin,
    agrees: bool,
) -> FieldAgreement {
    FieldAgreement {
        field,
        expectation,
        observation,
        agrees,
    }
}

/// Which recomputation step a wire refusal belongs to.
fn step_of(refusal: ConfidentialFundingRefusal) -> FundingRecordRefusal {
    match refusal {
        ConfidentialFundingRefusal::ResponseArmMismatch { .. } => {
            FundingRecordRefusal::Arm { refusal }
        }
        ConfidentialFundingRefusal::ResponseFixtureBindingMismatch
        | ConfidentialFundingRefusal::ResponseProfileBindingMismatch
        | ConfidentialFundingRefusal::UnknownFixtureHandle { .. }
        | ConfidentialFundingRefusal::FixtureDigestMismatch { .. } => {
            FundingRecordRefusal::Fixture { refusal }
        }
        _ => FundingRecordRefusal::Readback { refusal },
    }
}

/// Builds one validated record by recomputing every member in order.
///
/// # The eight recomputations, each with its own refusal
///
/// The arms match and no fallback occurred; the handle, digest,
/// profiles, request, response, and transaction all bind; the output
/// order is the fixture's; each output is exactly the hybrid tuple; the
/// commitment prefixes are the admitted pair; the proof shape comes from
/// the readback; submission, mining, readback, and outputs agree; and
/// the summary recomputes from the members rather than being carried.
/// The independent commitment comparison and the canonical boundary
/// carry their own refusals beside those eight.
///
/// # Errors
///
/// [`FundingRecordRefusal`] naming the step that did not recompute.
#[allow(clippy::too_many_lines)]
pub fn validate_confidential_funding_record(
    evidence: &ConfidentialFundingEvidence<'_>,
    oracles: &ConfidentialFundingOracles<'_>,
) -> Result<ValidatedConfidentialFundingRecord, FundingRecordRefusal> {
    let advertisement =
        evidence
            .handshake
            .confidential_funding
            .as_ref()
            .ok_or(FundingRecordRefusal::Fixture {
                refusal: ConfidentialFundingRefusal::ConfidentialFundingCapabilityAbsent,
            })?;
    let exchange = ConfidentialFundingExchange {
        requested: evidence.request,
        recorded: &evidence.request.binding,
        advertisement,
    };

    // Steps one, two, and seven: the arms, the bindings, and the whole
    // readback, each mapped to the step it belongs to.
    let observation =
        bind_confidential_funding(&exchange, evidence.response, oracles).map_err(step_of)?;
    let readback = match observation {
        ConfidentialFundingObservation::NoFundedObservation => {
            return Err(FundingRecordRefusal::NoFundedObservation);
        }
        ConfidentialFundingObservation::Funded(funding) => *funding,
    };

    // The record binds the registry's own case, and the request's handle
    // and digest must be that case rather than another registered one.
    if evidence.request.binding.fixture_handle != *evidence.fixture.handle()
        || evidence.request.binding.fixture_digest != *evidence.fixture.digest()
    {
        return Err(FundingRecordRefusal::Fixture {
            refusal: ConfidentialFundingRefusal::ResponseFixtureBindingMismatch,
        });
    }
    if evidence.request.binding.profiles != *evidence.fixture.profiles() {
        return Err(FundingRecordRefusal::Fixture {
            refusal: ConfidentialFundingRefusal::ResponseProfileBindingMismatch,
        });
    }

    // A record under recorded randomness may never claim deterministic
    // bytes, and a byte comparison is exactly that claim.
    if evidence.materialized_bytes_compared
        && evidence.fixture.contract() != ReproducibilityContract::ByteIdentity
    {
        return Err(FundingRecordRefusal::ReproducibilityClaimMismatch);
    }

    // Step three: the output order is the fixture's fixed order, in
    // count and in role.
    let fixture_outputs = evidence.fixture.outputs();
    if fixture_outputs.len() != readback.outputs.len()
        || fixture_outputs.len() != evidence.request.destinations.len()
    {
        return Err(FundingRecordRefusal::Order);
    }
    for (index, expected) in fixture_outputs.iter().enumerate() {
        let observed = &readback.outputs[index];
        if expected.output_program != observed.output_program {
            return Err(FundingRecordRefusal::Order);
        }
        // The protocol region occupies the first positions of the mined
        // transaction, in fixture order. Without this the classifier
        // below would be reading some other member's bytes at this
        // index and calling them this output's.
        if usize::try_from(observed.outpoint.vout).unwrap_or(usize::MAX) != index {
            return Err(FundingRecordRefusal::Order);
        }
    }

    // The non-protocol region, classified exhaustively over the whole
    // decoded transaction rather than over the protocol members alone.
    let protocol_asset = readback
        .outputs
        .first()
        .map(|output| output.explicit_asset.clone())
        .ok_or(FundingRecordRefusal::Order)?;
    let regions =
        classify_funding_members(evidence.decoded, &protocol_asset, fixture_outputs.len())
            .map_err(|refusal| FundingRecordRefusal::Region { refusal })?;

    let openings = match evidence.fixture.openings() {
        FixtureOpenings::Derived { openings, .. } => openings,
        FixtureOpenings::RunProduced => {
            return Err(FundingRecordRefusal::ReproducibilityClaimMismatch);
        }
    };

    let reported_witness_id = evidence
        .response
        .mined_readback
        .as_ref()
        .map(|readback| readback.witness_transaction_id.clone())
        .ok_or(FundingRecordRefusal::Readback {
            refusal: ConfidentialFundingRefusal::MinedReadbackMismatch {
                fact: crate::confidential_funding::MinedReadbackFact::ReadbackAbsent,
                index: None,
            },
        })?;

    let value_encoding = target_elements::reviewed_confidential_review_facts().value();
    let (low_prefix, high_prefix) = value_encoding.committed_prefixes();
    let required_prefixes = [low_prefix, high_prefix];

    let mut outputs = Vec::with_capacity(fixture_outputs.len());
    let mut agreement = Vec::with_capacity(fixture_outputs.len());
    let mut checks = Vec::with_capacity(fixture_outputs.len());
    for (index, expected) in fixture_outputs.iter().enumerate() {
        let observed = &readback.outputs[index];
        let reported = &evidence.response.confidential_funded_outputs[index];
        let decoded = evidence
            .decoded
            .outputs
            .get(index)
            .ok_or(FundingRecordRefusal::Order)?;

        // Step four: exactly the hybrid tuple.
        let form = form_census(decoded, observed, &protocol_asset, oracles.rangeproofs);
        if form != FundingOutputForm::complete() {
            return Err(FundingRecordRefusal::Representation { output: index });
        }

        // Step five: the admitted prefix pair, in fixed order, read from
        // the reviewed target contract rather than from a literal.
        let observation_commitment =
            ReadBackCommitment::from_readback(observed).map_err(|ReadBackWidthRefused| {
                FundingRecordRefusal::Representation { output: index }
            })?;
        if !value_encoding.admits_prefix(observation_commitment.prefix())
            || observation_commitment.prefix() != required_prefixes[index]
        {
            return Err(FundingRecordRefusal::PrefixMask { output: index });
        }

        // Step six: the proof shape, read from the readback and verified
        // against THIS output's commitment, generator, and program.
        if !oracles.rangeproofs.verifies(
            &observed.rangeproof,
            observation_commitment.bytes(),
            &observed.explicit_asset,
            &observed.output_program,
        ) || !decoded.surjection_proof.is_empty()
        {
            return Err(FundingRecordRefusal::ProofShape { output: index });
        }

        // The independent comparison: first-party arithmetic over the
        // registered opening against the bytes the target mined.
        let recomputation = RecomputedCommitment::recompute(
            evidence.fixture.explicit_asset(),
            expected.semantic_amount,
            &openings[index].value_blinder,
        )
        .map_err(|_| FundingRecordRefusal::CommitmentCheck { output: index })?;
        let check = compare_commitments(recomputation, observation_commitment);
        if !check.agrees() {
            return Err(FundingRecordRefusal::CommitmentCheck { output: index });
        }
        checks.push(check);

        let census = OutputAgreementCensus::assemble(
            index,
            field_agreements(
                expected,
                observed,
                reported,
                &recomputation,
                &observation_commitment,
                &openings[index].nonce_input,
                &WitnessIdentityBinding {
                    reported: &reported_witness_id,
                    recomputed: &readback.witness_transaction_id,
                    transaction_id: &readback.transaction_id,
                },
            ),
        )
        .map_err(|refusal| FundingRecordRefusal::Agreement { refusal })?;
        census
            .require_agreement()
            .map_err(|refusal| FundingRecordRefusal::Agreement { refusal })?;
        agreement.push(census);

        outputs.push(ValidatedFundingOutput {
            output_index: index,
            role: expected.role,
            outpoint: observed.outpoint.clone(),
            explicit_asset: observed.explicit_asset.clone(),
            value_commitment: observed.value_commitment.clone(),
            commitment_prefix: observation_commitment.prefix(),
            nonce: observed.nonce.clone(),
            output_program: observed.output_program.clone(),
            rangeproof: observed.rangeproof.clone(),
            surjection_proof: decoded.surjection_proof.clone(),
            witness_transaction_id: readback.witness_transaction_id.clone(),
            form,
        });
    }

    // Step eight: the summary recomputes from the members.
    let summary = FundingSummary {
        protocol_outputs: outputs.len(),
        non_protocol_members: regions
            .iter()
            .filter(|region| matches!(region, FundingRegion::NonProtocol(_)))
            .count(),
        agreeing_fields: agreement
            .iter()
            .map(|census| {
                census
                    .fields()
                    .iter()
                    .filter(|entry| entry.agrees())
                    .count()
            })
            .sum(),
        agreeing_commitments: checks.iter().filter(|check| check.agrees()).count(),
        observed_parities: outputs
            .iter()
            .map(ValidatedFundingOutput::commitment_prefix)
            .collect(),
        contract: evidence.fixture.contract(),
    };
    if summary.agreeing_fields != outputs.len() * FundingAgreementField::ALL.len()
        || summary.agreeing_commitments != outputs.len()
        || summary.observed_parities != required_prefixes[..outputs.len()]
    {
        return Err(FundingRecordRefusal::Summary);
    }

    Ok(ValidatedConfidentialFundingRecord {
        record: ConfidentialFundingRecord {
            handshake: evidence.handshake.clone(),
            environment: evidence.environment.clone(),
            request: evidence.request.clone(),
            response: evidence.response.clone(),
            fixture_handle: evidence.fixture.handle().clone(),
            fixture_digest: *evidence.fixture.digest(),
            profiles: *evidence.fixture.profiles(),
            readback,
            regions,
            outputs,
            agreement,
            independent_commitments: checks,
            observed_layer: evidence.response.observed_layer,
            exclusions: CanonicalFundingExclusion::ALL.into_iter().collect(),
            non_claims: CandidateFundingNonClaim::ALL.into_iter().collect(),
            summary,
        },
    })
}

/// The four form facts of one output, read from the decoding.
fn form_census(
    decoded: &DecodedFundingOutput,
    observed: &ReadbackDerivedOutput,
    protocol_asset: &str,
    verifier: &dyn RangeproofVerifier,
) -> BTreeSet<FundingOutputForm> {
    let mut census = BTreeSet::new();
    if matches!(decoded.value, DecodedValueField::Commitment(_)) {
        census.insert(FundingOutputForm::ConfidentialCommitment);
    }
    if matches!(&decoded.asset, DecodedAssetField::Explicit(asset) if asset == protocol_asset) {
        census.insert(FundingOutputForm::ExplicitProtocolAsset);
    }
    if !observed.rangeproof.is_empty()
        && verifier.verifies(
            &observed.rangeproof,
            &observed.value_commitment,
            &observed.explicit_asset,
            &observed.output_program,
        )
    {
        census.insert(FundingOutputForm::ValidRangeproof);
    }
    if decoded.surjection_proof.is_empty() {
        census.insert(FundingOutputForm::EmptySurjectionProof);
    }
    census
}

/// What a witness-transaction identity is checked against.
///
/// Three values rather than one, because the census member is a binding
/// and not a string comparison: the identity the answer reported, the
/// identity recomputed from the witness-bearing bytes, and the
/// witnessless identity those bytes must NOT reduce to. An agreement
/// recorded against a non-witness identity cannot be mistaken for
/// agreement about the proofs, because the third value is what rules it
/// out.
struct WitnessIdentityBinding<'a> {
    /// The identity the answer reported.
    reported: &'a str,
    /// The identity recomputed from the witness-bearing bytes.
    recomputed: &'a str,
    /// The witnessless identity of the same transaction.
    transaction_id: &'a str,
}

impl WitnessIdentityBinding<'_> {
    /// Whether the mined index and the witness-bearing bytes bind.
    fn binds(&self, output_witness_index: u32, vout: u32) -> bool {
        output_witness_index == vout
            && self.reported == self.recomputed
            && self.recomputed != self.transaction_id
    }
}

/// One output's eight field agreements, each between two origins.
fn field_agreements(
    expected: &ConfidentialFixtureOutput,
    observed: &ReadbackDerivedOutput,
    reported: &ConfidentialFundedOutput,
    recomputation: &RecomputedCommitment,
    observation: &ReadBackCommitment,
    nonce_input: &[u8; 32],
    witness: &WitnessIdentityBinding<'_>,
) -> Vec<FieldAgreement> {
    let nonce = recomputed_nonce(nonce_input);
    vec![
        agreement(
            FundingAgreementField::Asset,
            AgreementOrigin::Materialized,
            AgreementOrigin::ReadBack,
            reported.explicit_asset == observed.explicit_asset,
        ),
        agreement(
            FundingAgreementField::Commitment,
            AgreementOrigin::Recomputed,
            AgreementOrigin::ReadBack,
            recomputation.bytes() == observation.bytes(),
        ),
        agreement(
            FundingAgreementField::Parity,
            AgreementOrigin::Recomputed,
            AgreementOrigin::ReadBack,
            recomputation.prefix() == observation.prefix(),
        ),
        agreement(
            FundingAgreementField::Nonce,
            AgreementOrigin::Recomputed,
            AgreementOrigin::ReadBack,
            nonce.is_some_and(|encoded| encoded.as_slice() == observed.nonce.as_slice()),
        ),
        agreement(
            FundingAgreementField::Program,
            AgreementOrigin::Recomputed,
            AgreementOrigin::ReadBack,
            expected.output_program == observed.output_program,
        ),
        agreement(
            FundingAgreementField::ProofShape,
            AgreementOrigin::Reference,
            AgreementOrigin::ReadBack,
            !observed.rangeproof.is_empty() && reported.surjection_proof.is_empty(),
        ),
        agreement(
            FundingAgreementField::Outpoint,
            AgreementOrigin::Materialized,
            AgreementOrigin::ReadBack,
            reported.outpoint == observed.outpoint,
        ),
        agreement(
            FundingAgreementField::WitnessTransactionIdentity,
            AgreementOrigin::Materialized,
            AgreementOrigin::ReadBack,
            witness.binds(reported.output_witness_index, observed.outpoint.vout),
        ),
    ]
}
