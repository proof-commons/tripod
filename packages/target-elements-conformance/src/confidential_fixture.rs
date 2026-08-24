//! The confidential fixture registry, its handle grammar, its drift
//! digest, and its deterministic derivation recipe.
//!
//! # The registry is the only thing with lookup authority
//!
//! No environment value, no wallet, no request member other than the
//! handle and the digest, and no response echo may resolve a fixture.
//! Registration happens before the freeze and lookup only after it, and
//! the freeze is a type transition rather than a flag: a
//! [`ConfidentialFixtureRegistry`] is consumed to produce a
//! [`FrozenConfidentialFixtureRegistry`], so a late write is not a rule
//! that could be broken but a value that no longer exists.
//!
//! # Everything here is public disposable test material
//!
//! Every blinder, nonce input, and proof seed this module derives is a
//! published test fixture belonging to a disposable chain
//! `(ADR-015 rule test-material)`. The class is a typed member of the
//! manifest — [`PublicDisposableTestMaterial`] — rather than a claim in
//! a doc comment, and nothing here generates, retains, or transports a
//! production secret. A value that is secret-shaped is not a secret; it
//! is a constant this repository publishes.
//!
//! # Both reproducibility contracts are carried, and neither is inferred
//!
//! Under [`ReproducibilityContract::ByteIdentity`] the registry derives
//! every scalar the ceremony needs and the digest binds them. Under
//! [`ReproducibilityContract::RecordedRandomness`] the openings do not
//! exist until the run produces them, so the manifest carries none and
//! the digest binds the semantic transcript alone. The contract tag sits
//! INSIDE the transcript rather than beside it, which is what makes a
//! semantic-only digest impossible to mistake for a byte-identity one.
//!
//! # Nothing here reaches a target, and nothing here is a verdict
//!
//! Every refusal this module mints is a construction refusal. A
//! fixture-lookup failure is not evidence about a chain, an exhausted
//! counter is not a target rejection, and a diagnostic may name a
//! handle, a digest, a role, a counter, and a refusal kind and may name
//! nothing else.

use std::collections::BTreeMap;

use num_bigint::BigUint;
use num_traits::Zero as _;

use target_elements::ReproducibilityContract;

use crate::commitment_oracle::commitment::{
    self, CommitmentDefect, SCALAR_BYTES, is_semantic_amount,
};
use crate::commitment_oracle::curve;
use crate::confidential_funding::{
    ConfidentialFixtureResolution, FixtureResolutionRefused, wire_tag,
};
use crate::constructor::tagged::tagged_hash;
use crate::protocol::{
    ConfidentialFixtureDigest, ConfidentialFixtureHandle, ConfidentialFundingProfiles,
};

/// The grammar version every handle this module admits carries.
///
/// The one digit in the whole spelling. A later grammar is a different
/// prefix rather than a reinterpretation of this one.
pub const HANDLE_GRAMMAR_VERSION: u16 = 1;

/// The prefix every admitted handle begins with.
pub const HANDLE_PREFIX: &str = "ctf-v1/";

/// The shortest admitted case name.
pub const MINIMUM_CASE_NAME: usize = 3;

/// The longest admitted case name.
pub const MAXIMUM_CASE_NAME: usize = 63;

/// The tag the fixture digest is taken under.
pub const FIXTURE_DIGEST_TAG: &str = "tripod/guide-ctf/fixture-digest/v1";

/// The tag every derived scalar is taken under.
pub const DERIVATION_TAG: &str = "tripod/guide-ctf/derive/v1";

/// The highest parity counter the search may reach.
///
/// A contract constant rather than a tuning knob: search moves upward
/// from zero without wrapping, without skipping, without randomness, and
/// without concurrency, and exhaustion is a typed refusal rather than a
/// retry with a different source.
pub const MAX_PARITY_COUNTER: u16 = 4095;

/// The highest scalar counter one role's search may reach.
pub const MAX_SCALAR_COUNTER: u8 = 255;

/// The width of a derived scalar or seed.
pub const DERIVED_BYTES: usize = 32;

/// The width of a serialized value commitment, prefix included.
pub const COMMITMENT_BYTES: usize = 33;

/// The class every value in this registry belongs to.
///
/// This guide's name for the class ADR-015's test-material rule already
/// carries, stated as a type so that a manifest declares it rather than
/// a comment asserting it. The construction side states the same fact as
/// `PrivateConstructionNonClaim::NoOpeningIsSecret`, and the two are
/// held equal by a test rather than by intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PublicDisposableTestMaterial {
    /// ADR-015's disposable test-network material.
    DisposableTestNetworkMaterial,
}

impl PublicDisposableTestMaterial {
    /// The only class this registry admits.
    pub const EXPECTED: Self = Self::DisposableTestNetworkMaterial;

    /// The class's transcript code.
    #[must_use]
    pub const fn transcript_code(self) -> u8 {
        match self {
            Self::DisposableTestNetworkMaterial => 1,
        }
    }
}

/// What one output of a fixture is for.
///
/// Exhaustive and disjoint: exactly one output of a transaction is
/// `Balancing` and every other is `Primary`. The balancing blinder is
/// solved rather than derived, which is the general rule the first
/// predecessor's ordered additive inverses are one instance of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FixtureOutputRole {
    /// An output whose value blinder is derived.
    Primary,
    /// The one output whose value blinder is solved from the others.
    Balancing,
}

impl FixtureOutputRole {
    /// The role's transcript code.
    #[must_use]
    pub const fn transcript_code(self) -> u8 {
        match self {
            Self::Primary => 1,
            Self::Balancing => 2,
        }
    }
}

impl std::fmt::Display for FixtureOutputRole {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Primary => "primary",
            Self::Balancing => "balancing",
        };
        formatter.write_str(text)
    }
}

/// Which scalar a derivation preimage is for.
///
/// Domain separation by role as well as by case, so that no two
/// preimages can collide by concatenation and no role's value can stand
/// in for another's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DerivationRole {
    /// The output's value blinder.
    ValueBlinder,
    /// The output's nonce input.
    NonceSecret,
    /// The output's rangeproof seed.
    RangeproofSeed,
}

impl DerivationRole {
    /// Every role, in the order the vocabulary states them.
    pub const ALL: [Self; 3] = [Self::ValueBlinder, Self::NonceSecret, Self::RangeproofSeed];

    /// The role's derivation code.
    #[must_use]
    pub const fn derivation_code(self) -> u8 {
        match self {
            Self::ValueBlinder => 1,
            Self::NonceSecret => 2,
            Self::RangeproofSeed => 3,
        }
    }

    /// Whether the role's value is read as a scalar.
    ///
    /// A rangeproof seed uses all thirty-two hash bytes and is not a
    /// scalar; the other two are read as big-endian scalars and must be
    /// nonzero and below the group order.
    #[must_use]
    pub const fn is_scalar(self) -> bool {
        match self {
            Self::ValueBlinder | Self::NonceSecret => true,
            Self::RangeproofSeed => false,
        }
    }
}

impl std::fmt::Display for DerivationRole {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::ValueBlinder => "value blinder",
            Self::NonceSecret => "nonce input",
            Self::RangeproofSeed => "rangeproof seed",
        };
        formatter.write_str(text)
    }
}

/// The derivation recipe this guide's fixtures are built under.
///
/// Candidate vocabulary. The generation in the spelling is the recipe's
/// own and is not a release identity, an architecture operation, or a
/// schema identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FixtureDerivationProfile {
    /// This guide's deterministic recipe.
    GuideCtfV1,
}

impl FixtureDerivationProfile {
    /// The tag this profile derives under.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::GuideCtfV1 => DERIVATION_TAG,
        }
    }

    /// The profile's transcript code.
    #[must_use]
    pub const fn transcript_code(self) -> u8 {
        match self {
            Self::GuideCtfV1 => 1,
        }
    }
}

/// Why a handle's spelling is not one the grammar admits.
///
/// Closed, with no catch-all, and every variant names a position or a
/// width rather than quoting the spelling back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum HandleGrammarDefect {
    /// The spelling does not begin with the grammar's prefix.
    PrefixAbsent,
    /// The case name is shorter than the grammar admits.
    CaseNameTooShort {
        /// The width found.
        found: usize,
    },
    /// The case name is longer than the grammar admits.
    CaseNameTooLong {
        /// The width found.
        found: usize,
    },
    /// A byte the grammar does not admit.
    ///
    /// Lowercase letters and interior hyphens, and nothing else. The one
    /// digit in the whole spelling belongs to the prefix.
    ByteNotAdmitted {
        /// The position within the case name.
        position: usize,
    },
    /// The case name begins with a hyphen.
    HyphenAtStart,
    /// The case name ends with something other than a letter.
    HyphenAtEnd,
    /// Two hyphens sit beside each other.
    AdjacentHyphens {
        /// The position of the second hyphen.
        position: usize,
    },
}

impl std::fmt::Display for HandleGrammarDefect {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PrefixAbsent => {
                write!(
                    formatter,
                    "the spelling does not begin with {HANDLE_PREFIX}"
                )
            }
            Self::CaseNameTooShort { found } => write!(
                formatter,
                "the case name is {found} bytes and the grammar admits at least {MINIMUM_CASE_NAME}",
            ),
            Self::CaseNameTooLong { found } => write!(
                formatter,
                "the case name is {found} bytes and the grammar admits at most {MAXIMUM_CASE_NAME}",
            ),
            Self::ByteNotAdmitted { position } => {
                write!(
                    formatter,
                    "the case name carries an inadmissible byte at {position}"
                )
            }
            Self::HyphenAtStart => formatter.write_str("the case name begins with a hyphen"),
            Self::HyphenAtEnd => formatter.write_str("the case name does not end in a letter"),
            Self::AdjacentHyphens { position } => {
                write!(
                    formatter,
                    "the case name carries adjacent hyphens at {position}"
                )
            }
        }
    }
}

/// Whether one spelling is a handle the grammar admits.
///
/// The grammar belongs to the registry, which is why the check lives
/// here and not on the wire type: what travels on the wire is a
/// spelling, and what this function answers is whether the registry
/// would ever have registered one.
///
/// # Errors
///
/// [`HandleGrammarDefect`] at the first clause the spelling fails.
pub fn check_handle_grammar(handle: &ConfidentialFixtureHandle) -> Result<(), HandleGrammarDefect> {
    let spelling = handle.as_str();
    let case_name = spelling
        .strip_prefix(HANDLE_PREFIX)
        .ok_or(HandleGrammarDefect::PrefixAbsent)?;
    let bytes = case_name.as_bytes();
    if bytes.len() < MINIMUM_CASE_NAME {
        return Err(HandleGrammarDefect::CaseNameTooShort { found: bytes.len() });
    }
    if bytes.len() > MAXIMUM_CASE_NAME {
        return Err(HandleGrammarDefect::CaseNameTooLong { found: bytes.len() });
    }
    for (position, byte) in bytes.iter().enumerate() {
        match byte {
            b'a'..=b'z' => {}
            b'-' => {
                if position == 0 {
                    return Err(HandleGrammarDefect::HyphenAtStart);
                }
                if position + 1 == bytes.len() {
                    return Err(HandleGrammarDefect::HyphenAtEnd);
                }
                if bytes[position - 1] == b'-' {
                    return Err(HandleGrammarDefect::AdjacentHyphens { position });
                }
            }
            _ => return Err(HandleGrammarDefect::ByteNotAdmitted { position }),
        }
    }
    Ok(())
}

/// One output as the manifest states it.
///
/// The semantic facts and nothing derived. What the registry adds to
/// these is the openings, and under recorded randomness it adds none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfidentialFixtureOutput {
    /// What this output is for.
    pub role: FixtureOutputRole,
    /// The semantic amount this output carries.
    pub semantic_amount: u64,
    /// The witness program this output pays.
    pub output_program: Vec<u8>,
}

/// The complete public description of one case.
///
/// It carries semantics and never openings: the openings are the
/// registry's own derivation under byte identity, and the run's own
/// under recorded randomness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfidentialFixtureManifest {
    /// The case identity.
    pub handle: ConfidentialFixtureHandle,
    /// The class every value of this case belongs to.
    pub material_class: PublicDisposableTestMaterial,
    /// The recipe the openings are derived under.
    pub derivation_profile: FixtureDerivationProfile,
    /// The profiles the ceremony selects.
    pub profiles: ConfidentialFundingProfiles,
    /// The parity counter bound this case searches under.
    pub retry_limit: u16,
    /// The explicit protocol asset every protocol output carries.
    pub explicit_asset: [u8; 32],
    /// The value blinder the funding inputs contribute, as a scalar.
    ///
    /// Zero for the first predecessor, whose one explicit input
    /// contributes a zero value blinder. It is a member rather than a
    /// constant because the balancing solve is stated once, generally,
    /// and the zero case is one instance of it.
    pub input_blinder_sum: [u8; 32],
    /// The outputs, in the fixed order the fixture fixes.
    pub outputs: Vec<ConfidentialFixtureOutput>,
}

/// One output's derived openings.
///
/// Public disposable test material in every member. Nothing here is a
/// secret and nothing here may be described as one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DerivedOpening {
    /// The output's value blinder.
    pub value_blinder: [u8; DERIVED_BYTES],
    /// The output's nonce input.
    pub nonce_input: [u8; DERIVED_BYTES],
    /// The output's rangeproof seed.
    pub rangeproof_seed: [u8; DERIVED_BYTES],
    /// The serialized value commitment the opening produces.
    pub value_commitment: [u8; COMMITMENT_BYTES],
}

/// The openings one resolved fixture carries.
///
/// Two shapes because the two contracts genuinely differ here and
/// nowhere else: byte identity derives every scalar before the run, and
/// recorded randomness has none until the run produces them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FixtureOpenings {
    /// Openings derived by the registry, with the counter that found
    /// them.
    Derived {
        /// The parity counter the search settled on.
        parity_counter: u16,
        /// One opening per output, in fixture order.
        openings: Vec<DerivedOpening>,
    },
    /// No openings, because the run has not produced them yet.
    RunProduced,
}

/// One registered case, resolved.
///
/// Ordered roles, amounts, programs, asset, order, and openings, and
/// nothing a request could have supplied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedFixture {
    handle: ConfidentialFixtureHandle,
    digest: ConfidentialFixtureDigest,
    material_class: PublicDisposableTestMaterial,
    derivation_profile: FixtureDerivationProfile,
    profiles: ConfidentialFundingProfiles,
    retry_limit: u16,
    explicit_asset: [u8; 32],
    input_blinder_sum: [u8; 32],
    outputs: Vec<ConfidentialFixtureOutput>,
    openings: FixtureOpenings,
}

impl ResolvedFixture {
    /// The case identity.
    #[must_use]
    pub const fn handle(&self) -> &ConfidentialFixtureHandle {
        &self.handle
    }

    /// The digest this case was registered under.
    #[must_use]
    pub const fn digest(&self) -> &ConfidentialFixtureDigest {
        &self.digest
    }

    /// The class every value of this case belongs to.
    #[must_use]
    pub const fn material_class(&self) -> PublicDisposableTestMaterial {
        self.material_class
    }

    /// The recipe the openings are derived under.
    #[must_use]
    pub const fn derivation_profile(&self) -> FixtureDerivationProfile {
        self.derivation_profile
    }

    /// The profiles this case selects.
    #[must_use]
    pub const fn profiles(&self) -> &ConfidentialFundingProfiles {
        &self.profiles
    }

    /// The contract this case runs under.
    #[must_use]
    pub const fn contract(&self) -> ReproducibilityContract {
        self.profiles.reproducibility_contract
    }

    /// The parity counter bound this case searched under.
    #[must_use]
    pub const fn retry_limit(&self) -> u16 {
        self.retry_limit
    }

    /// The explicit protocol asset every protocol output carries.
    #[must_use]
    pub const fn explicit_asset(&self) -> &[u8; 32] {
        &self.explicit_asset
    }

    /// The value blinder the funding inputs contribute.
    #[must_use]
    pub const fn input_blinder_sum(&self) -> &[u8; 32] {
        &self.input_blinder_sum
    }

    /// The outputs, in the fixed order the fixture fixes.
    #[must_use]
    pub fn outputs(&self) -> &[ConfidentialFixtureOutput] {
        &self.outputs
    }

    /// The openings, which only the frozen registry supplies.
    #[must_use]
    pub const fn openings(&self) -> &FixtureOpenings {
        &self.openings
    }

    /// The serialized value commitments, in fixture order.
    ///
    /// Absent under recorded randomness, where the openings do not exist
    /// until the run produces them.
    #[must_use]
    pub fn value_commitments(&self) -> Option<Vec<[u8; COMMITMENT_BYTES]>> {
        match &self.openings {
            FixtureOpenings::Derived { openings, .. } => Some(
                openings
                    .iter()
                    .map(|opening| opening.value_commitment)
                    .collect(),
            ),
            FixtureOpenings::RunProduced => None,
        }
    }
}

/// Why one derivation did not produce a fixture.
///
/// Closed, with no catch-all. The last variant carries the rule that
/// matters most: a proof failure never becomes a source of entropy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FixtureDerivationRefusal {
    /// One role's scalar counter reached its bound.
    ScalarSearchExhausted {
        /// Which role.
        role: DerivationRole,
        /// How many counters were tried.
        attempts: u16,
    },
    /// The solved balancing blinder is zero or out of range.
    DegenerateBalancingScalar,
    /// The independent recheck of the two sums disagreed.
    BlinderBalanceMismatch,
    /// A commitment landed on the group identity.
    IdentityValueCommitment {
        /// Which output.
        output: usize,
    },
    /// The parity counter reached its bound without the required pair.
    ParitySearchExhausted {
        /// How many counters were tried.
        attempts: u32,
    },
    /// Proof generation refused; no randomness is added and no retry
    /// follows.
    RangeproofConstructionFailed {
        /// Which output.
        output: usize,
    },
}

impl std::fmt::Display for FixtureDerivationRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ScalarSearchExhausted { role, attempts } => write!(
                formatter,
                "the {role} search reached its bound after {attempts} counters",
            ),
            Self::DegenerateBalancingScalar => {
                formatter.write_str("the solved balancing blinder is zero or out of range")
            }
            Self::BlinderBalanceMismatch => {
                formatter.write_str("the independent recheck of the blinder sums disagreed")
            }
            Self::IdentityValueCommitment { output } => {
                write!(formatter, "output {output} commits to the group identity")
            }
            Self::ParitySearchExhausted { attempts } => write!(
                formatter,
                "the parity search reached its bound after {attempts} counters",
            ),
            Self::RangeproofConstructionFailed { output } => {
                write!(formatter, "proof generation refused output {output}")
            }
        }
    }
}

/// Why one manifest was not registered.
///
/// Closed, with no catch-all, and every variant a construction refusal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RegistrationRefusal {
    /// The handle's spelling is not one the grammar admits.
    HandleGrammar {
        /// The clause it failed.
        defect: HandleGrammarDefect,
    },
    /// A fixture is already registered under this handle.
    ///
    /// Refused even where the two manifests are equal: a registry that
    /// silently accepted an equal re-registration would accept an
    /// unequal one on the day the two drifted.
    DuplicateFixtureHandle,
    /// The manifest declares a class this registry does not admit.
    MaterialClassNotAdmitted,
    /// The manifest states fewer outputs than a balance needs.
    OutputSetTooSmall {
        /// How many outputs were stated.
        found: usize,
    },
    /// The roles are not exhaustive and disjoint.
    ///
    /// Exactly one output is balancing. Zero leaves the blinder sum
    /// unsolvable and two leaves it underdetermined.
    BalancingRoleNotUnique {
        /// How many balancing outputs were stated.
        found: usize,
    },
    /// An output states an amount outside the semantic domain.
    AmountNotSemantic {
        /// Which output.
        output: usize,
    },
    /// An output states a nonpositive amount.
    AmountNotPositive {
        /// Which output.
        output: usize,
    },
    /// An output states no program.
    OutputProgramEmpty {
        /// Which output.
        output: usize,
    },
    /// The stated retry limit is above the contract's bound.
    RetryLimitAboveBound {
        /// The limit stated.
        stated: u16,
    },
    /// The stated input blinder sum is not an admitted scalar.
    InputBlinderSumInvalid,
    /// The derivation refused.
    Derivation {
        /// Its own typed cause.
        refusal: FixtureDerivationRefusal,
    },
}

impl std::fmt::Display for RegistrationRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HandleGrammar { defect } => {
                write!(formatter, "the handle is inadmissible: {defect}")
            }
            Self::DuplicateFixtureHandle => {
                formatter.write_str("a fixture is already registered under this handle")
            }
            Self::MaterialClassNotAdmitted => {
                formatter.write_str("the manifest declares a class this registry does not admit")
            }
            Self::OutputSetTooSmall { found } => {
                write!(formatter, "the manifest states {found} outputs")
            }
            Self::BalancingRoleNotUnique { found } => {
                write!(formatter, "the manifest states {found} balancing outputs")
            }
            Self::AmountNotSemantic { output } => {
                write!(
                    formatter,
                    "output {output} states an amount outside the semantic domain"
                )
            }
            Self::AmountNotPositive { output } => {
                write!(formatter, "output {output} states a nonpositive amount")
            }
            Self::OutputProgramEmpty { output } => {
                write!(formatter, "output {output} states no program")
            }
            Self::RetryLimitAboveBound { stated } => write!(
                formatter,
                "the stated retry limit {stated} is above the bound {MAX_PARITY_COUNTER}",
            ),
            Self::InputBlinderSumInvalid => {
                formatter.write_str("the stated input blinder sum is not an admitted scalar")
            }
            Self::Derivation { refusal } => write!(formatter, "the derivation refused: {refusal}"),
        }
    }
}

/// What a fixture diagnostic may say.
///
/// The refusal's own kind, and never its payload's amounts. The
/// vocabulary is closed so that a diagnostic naming a fault cannot name
/// one the registry does not have.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FixtureDiagnosticKind {
    /// Registration refused.
    RegistrationRefused,
    /// Derivation refused.
    DerivationRefused,
    /// Lookup found no such handle.
    HandleUnknown,
    /// Lookup found another digest.
    DigestDrifted,
}

impl std::fmt::Display for FixtureDiagnosticKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::RegistrationRefused => "registration refused",
            Self::DerivationRefused => "derivation refused",
            Self::HandleUnknown => "handle unknown",
            Self::DigestDrifted => "digest drifted",
        };
        formatter.write_str(text)
    }
}

/// What a fixture fault may be reported as.
///
/// Handle, digest, role, counter, and refusal kind, and nothing else: no
/// amount, no opening, no derivation material, no attachment, and no
/// free text. The absent members are absent as fields rather than
/// omitted by discipline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureDiagnostic {
    handle: ConfidentialFixtureHandle,
    digest: Option<ConfidentialFixtureDigest>,
    role: Option<FixtureOutputRole>,
    counter: Option<u16>,
    kind: FixtureDiagnosticKind,
}

impl FixtureDiagnostic {
    /// States one diagnostic.
    #[must_use]
    pub const fn new(
        handle: ConfidentialFixtureHandle,
        digest: Option<ConfidentialFixtureDigest>,
        role: Option<FixtureOutputRole>,
        counter: Option<u16>,
        kind: FixtureDiagnosticKind,
    ) -> Self {
        Self {
            handle,
            digest,
            role,
            counter,
            kind,
        }
    }

    /// The case the diagnostic is about.
    #[must_use]
    pub const fn handle(&self) -> &ConfidentialFixtureHandle {
        &self.handle
    }

    /// The digest, where one is known.
    #[must_use]
    pub const fn digest(&self) -> Option<&ConfidentialFixtureDigest> {
        self.digest.as_ref()
    }

    /// The role, where the fault belongs to one.
    #[must_use]
    pub const fn role(&self) -> Option<FixtureOutputRole> {
        self.role
    }

    /// The counter, where the fault reached one.
    #[must_use]
    pub const fn counter(&self) -> Option<u16> {
        self.counter
    }

    /// What kind of fault it was.
    #[must_use]
    pub const fn kind(&self) -> FixtureDiagnosticKind {
        self.kind
    }
}

impl std::fmt::Display for FixtureDiagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.handle, self.kind)?;
        if let Some(role) = self.role {
            write!(formatter, ", role {role}")?;
        }
        if let Some(counter) = self.counter {
            write!(formatter, ", counter {counter}")?;
        }
        Ok(())
    }
}

/// A transcript whose every member carries its own width.
///
/// Big-endian integers with explicit lengths, so that no two transcripts
/// can collide by concatenation. It is an ephemeral hash preimage and a
/// plain typed value; storing or exchanging it would change its form.
#[derive(Clone, Debug, Default)]
struct FramedTranscript {
    bytes: Vec<u8>,
}

impl FramedTranscript {
    /// An empty transcript.
    fn new() -> Self {
        Self::default()
    }

    /// One byte.
    fn octet(&mut self, value: u8) {
        self.bytes.push(value);
    }

    /// One sixteen-bit integer.
    fn word(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    /// One thirty-two-bit integer.
    fn long(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    /// One sixty-four-bit integer.
    fn quad(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    /// One byte string, framed by its own width.
    fn framed(&mut self, value: &[u8]) {
        let width = u32::try_from(value.len()).unwrap_or(u32::MAX);
        self.long(width);
        self.bytes.extend_from_slice(value);
    }

    /// The transcript's bytes.
    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

/// The digest transcript of one manifest under one set of openings.
///
/// The contract tag sits INSIDE the transcript rather than beside it,
/// which is what makes a semantic-only recorded-randomness digest
/// impossible to mistake for a byte-identity one, and it is why the
/// accepted semantic-only ruling costs one field rather than a second
/// digest.
fn digest_transcript(
    manifest: &ConfidentialFixtureManifest,
    openings: &FixtureOpenings,
) -> Vec<u8> {
    let mut transcript = FramedTranscript::new();
    transcript.word(HANDLE_GRAMMAR_VERSION);
    transcript.framed(manifest.handle.as_str().as_bytes());
    transcript.framed(manifest.profiles.reproducibility_contract.code().as_bytes());
    transcript.framed(wire_tag(&manifest.profiles.representation).as_bytes());
    transcript.framed(wire_tag(&manifest.profiles.custody).as_bytes());
    transcript.framed(wire_tag(&manifest.profiles.materializer).as_bytes());
    transcript.octet(manifest.derivation_profile.transcript_code());
    transcript.octet(manifest.material_class.transcript_code());
    transcript.word(manifest.retry_limit);
    match openings {
        FixtureOpenings::Derived { parity_counter, .. } => {
            transcript.octet(1);
            transcript.word(*parity_counter);
        }
        FixtureOpenings::RunProduced => transcript.octet(0),
    }
    let count = u32::try_from(manifest.outputs.len()).unwrap_or(u32::MAX);
    transcript.long(count);
    for (index, output) in manifest.outputs.iter().enumerate() {
        transcript.long(u32::try_from(index).unwrap_or(u32::MAX));
        transcript.octet(output.role.transcript_code());
        transcript.framed(&manifest.explicit_asset);
        transcript.framed(&output.output_program);
        if let FixtureOpenings::Derived { openings, .. } = openings {
            let opening = &openings[index];
            transcript.quad(output.semantic_amount);
            transcript.framed(&opening.value_blinder);
            transcript.framed(&opening.nonce_input);
            transcript.framed(&opening.rangeproof_seed);
            transcript.octet(opening.value_commitment[0]);
        }
    }
    transcript.finish()
}

/// One derivation preimage.
///
/// The case identity, the output index, the derivation role, the parity
/// counter, and the scalar counter, all framed, so that no two preimages
/// can collide by concatenation.
fn derivation_preimage(
    handle: &ConfidentialFixtureHandle,
    output: usize,
    role: DerivationRole,
    parity_counter: u16,
    scalar_counter: u8,
) -> Vec<u8> {
    let mut transcript = FramedTranscript::new();
    transcript.word(HANDLE_GRAMMAR_VERSION);
    transcript.framed(handle.as_str().as_bytes());
    transcript.long(u32::try_from(output).unwrap_or(u32::MAX));
    transcript.octet(role.derivation_code());
    transcript.word(parity_counter);
    transcript.octet(scalar_counter);
    transcript.finish()
}

/// Where a derivation's thirty-two bytes come from.
///
/// # Why a seam exists here at all
///
/// Six of [`FixtureDerivationRefusal`]'s variants are refusals about
/// derived material, and a deterministic recipe cannot be asked to
/// produce material that fails them: the tagged hash answers what it
/// answers, and no manifest a test may write changes the answer. So
/// those refusals were unreachable from any fixture, and a closed
/// vocabulary whose variants no test can reach is a vocabulary nobody
/// has checked.
///
/// The seam is the narrowest thing that fixes that. It is `pub(crate)`,
/// it sits between the preimage and its bytes, and it has exactly one
/// non-test implementation — [`TaggedHashDerivation`], which is the
/// recipe the profile's tag names and nothing else. No public surface
/// mentions it, no published digest changes because of it, and the
/// deterministic source is the only one anything outside this crate's
/// own tests can reach.
///
/// What it deliberately does NOT reach is the arithmetic. A source
/// chooses bytes; it does not choose what the commitment oracle makes of
/// them. That boundary is the point: the oracle is the workspace's
/// independence claim, and a seam that let a test replace it would have
/// made the claim testable by substitution.
pub(crate) trait DerivationSource {
    /// The thirty-two bytes this preimage derives to under `tag`.
    fn derive(&self, tag: &str, preimage: &[u8]) -> [u8; DERIVED_BYTES];
}

/// The deterministic source, and the only one outside this crate's
/// tests.
///
/// The tagged hash the selected profile names, called with the profile's
/// own tag. Substituting this is what the seam is for; changing it is
/// not, because its bytes are in every registered digest.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TaggedHashDerivation;

impl DerivationSource for TaggedHashDerivation {
    fn derive(&self, tag: &str, preimage: &[u8]) -> [u8; DERIVED_BYTES] {
        tagged_hash(tag, preimage)
    }
}

/// One derived value, before it is read as anything.
fn derive_bytes(
    source: &dyn DerivationSource,
    profile: FixtureDerivationProfile,
    handle: &ConfidentialFixtureHandle,
    output: usize,
    role: DerivationRole,
    parity_counter: u16,
    scalar_counter: u8,
) -> [u8; DERIVED_BYTES] {
    let preimage = derivation_preimage(handle, output, role, parity_counter, scalar_counter);
    source.derive(profile.tag(), &preimage)
}

/// The group order, once.
fn order() -> &'static BigUint {
    curve::group_order()
}

/// One scalar, as thirty-two big-endian bytes.
fn scalar_bytes(value: &BigUint) -> [u8; DERIVED_BYTES] {
    let mut bytes = [0_u8; DERIVED_BYTES];
    let raw = value.to_bytes_be();
    let width = raw.len().min(DERIVED_BYTES);
    bytes[DERIVED_BYTES - width..].copy_from_slice(&raw[raw.len() - width..]);
    bytes
}

/// One scalar's additive inverse in the group.
fn negate_scalar(value: &BigUint) -> BigUint {
    if value.is_zero() {
        BigUint::zero()
    } else {
        order() - value
    }
}

/// The sum of two scalars in the group.
fn add_scalars(left: &BigUint, right: &BigUint) -> BigUint {
    let sum = left + right;
    if &sum >= order() { sum - order() } else { sum }
}

/// Whether a derived value is a scalar the ceremony may use.
///
/// Nonzero and below the group order. The bound is not restated here:
/// [`commitment::read_scalar`] already implements exactly that reading
/// and its own defect vocabulary, and this reuses it.
fn admitted_scalar(bytes: &[u8; DERIVED_BYTES]) -> Option<BigUint> {
    let scalar = commitment::read_scalar(bytes).ok()?;
    if scalar.is_zero() { None } else { Some(scalar) }
}

/// One role's scalar for one output, searched upward from zero.
fn search_scalar(
    source: &dyn DerivationSource,
    profile: FixtureDerivationProfile,
    handle: &ConfidentialFixtureHandle,
    output: usize,
    role: DerivationRole,
    parity_counter: u16,
) -> Result<([u8; DERIVED_BYTES], BigUint), FixtureDerivationRefusal> {
    let mut counter = 0_u16;
    while counter <= u16::from(MAX_SCALAR_COUNTER) {
        let scalar_counter = u8::try_from(counter).unwrap_or(MAX_SCALAR_COUNTER);
        let bytes = derive_bytes(
            source,
            profile,
            handle,
            output,
            role,
            parity_counter,
            scalar_counter,
        );
        if let Some(scalar) = admitted_scalar(&bytes) {
            return Ok((bytes, scalar));
        }
        counter += 1;
    }
    Err(FixtureDerivationRefusal::ScalarSearchExhausted {
        role,
        attempts: u16::from(MAX_SCALAR_COUNTER) + 1,
    })
}

/// Every opening of one case, at one parity counter.
fn derive_at_counter(
    source: &dyn DerivationSource,
    manifest: &ConfidentialFixtureManifest,
    input_blinder_sum: &BigUint,
    parity_counter: u16,
) -> Result<Vec<DerivedOpening>, FixtureDerivationRefusal> {
    let profile = manifest.derivation_profile;
    let handle = &manifest.handle;

    let mut blinders: Vec<Option<[u8; DERIVED_BYTES]>> = Vec::with_capacity(manifest.outputs.len());
    let mut derived_sum = BigUint::zero();
    for (index, output) in manifest.outputs.iter().enumerate() {
        if output.role == FixtureOutputRole::Balancing {
            blinders.push(None);
            continue;
        }
        let (bytes, scalar) = search_scalar(
            source,
            profile,
            handle,
            index,
            DerivationRole::ValueBlinder,
            parity_counter,
        )?;
        derived_sum = add_scalars(&derived_sum, &scalar);
        blinders.push(Some(bytes));
    }

    // The balancing blinder is solved, never derived: the input blinder
    // sum minus the other outputs' sum, in the group. For the first
    // predecessor the input contributes zero and the two blinders come
    // out ordered additive inverses, which is that requirement as an
    // instance of the general rule rather than as a special case.
    let balancing = add_scalars(input_blinder_sum, &negate_scalar(&derived_sum));
    if balancing.is_zero() || &balancing >= order() {
        return Err(FixtureDerivationRefusal::DegenerateBalancingScalar);
    }
    let balancing_bytes = scalar_bytes(&balancing);
    for slot in &mut blinders {
        if slot.is_none() {
            *slot = Some(balancing_bytes);
        }
    }

    // The independent recheck, which is not the same statement as the
    // solve: the solve produced the value and this reads every blinder
    // back and adds them again.
    let mut recheck = BigUint::zero();
    for slot in blinders.iter().copied() {
        let bytes = slot.ok_or(FixtureDerivationRefusal::BlinderBalanceMismatch)?;
        let scalar = commitment::read_scalar(&bytes)
            .map_err(|_| FixtureDerivationRefusal::BlinderBalanceMismatch)?;
        recheck = add_scalars(&recheck, &scalar);
    }
    if recheck != *input_blinder_sum {
        return Err(FixtureDerivationRefusal::BlinderBalanceMismatch);
    }

    let mut openings = Vec::with_capacity(manifest.outputs.len());
    for (index, output) in manifest.outputs.iter().enumerate() {
        let value_blinder =
            blinders[index].ok_or(FixtureDerivationRefusal::DegenerateBalancingScalar)?;
        let (nonce_input, _) = search_scalar(
            source,
            profile,
            handle,
            index,
            DerivationRole::NonceSecret,
            parity_counter,
        )?;
        let rangeproof_seed = derive_bytes(
            source,
            profile,
            handle,
            index,
            DerivationRole::RangeproofSeed,
            parity_counter,
            0,
        );
        let value_commitment = commitment::commitment(
            &manifest.explicit_asset,
            output.semantic_amount,
            &value_blinder,
        )
        .map_err(|defect| match defect {
            CommitmentDefect::IdentityResult => {
                FixtureDerivationRefusal::IdentityValueCommitment { output: index }
            }
            CommitmentDefect::Scalar(_) | CommitmentDefect::Generator(_) => {
                FixtureDerivationRefusal::BlinderBalanceMismatch
            }
        })?;
        openings.push(DerivedOpening {
            value_blinder,
            nonce_input,
            rangeproof_seed,
            value_commitment,
        });
    }
    Ok(openings)
}

/// The prefix pair the search is looking for, in fixed order.
///
/// Read from the reviewed target contract rather than written here
/// twice, so a change in the target's admitted prefixes is a change in
/// one place.
fn required_prefixes() -> (u8, u8) {
    target_elements::reviewed_confidential_review_facts()
        .value()
        .committed_prefixes()
}

/// Whether one set of openings carries the required prefixes in fixed
/// order.
fn prefixes_match(openings: &[DerivedOpening]) -> bool {
    let required: [u8; 2] = required_prefixes().into();
    if openings.len() != required.len() {
        return false;
    }
    openings
        .iter()
        .zip(required)
        .all(|(opening, wanted)| opening.value_commitment[0] == wanted)
}

/// The bounded deterministic parity search.
///
/// Upward from zero, without wrapping, without skipping, without
/// randomness, and without concurrency. Exhaustion is a typed refusal
/// and never a retry with a different source.
fn search_parity(
    source: &dyn DerivationSource,
    manifest: &ConfidentialFixtureManifest,
    input_blinder_sum: &BigUint,
) -> Result<(u16, Vec<DerivedOpening>), FixtureDerivationRefusal> {
    let bound = manifest.retry_limit.min(MAX_PARITY_COUNTER);
    let mut counter = 0_u16;
    loop {
        let openings = derive_at_counter(source, manifest, input_blinder_sum, counter)?;
        if prefixes_match(&openings) {
            return Ok((counter, openings));
        }
        if counter >= bound {
            return Err(FixtureDerivationRefusal::ParitySearchExhausted {
                attempts: u32::from(bound) + 1,
            });
        }
        counter += 1;
    }
}

/// The registry, before the freeze.
///
/// Registration is possible only here, and lookup only after
/// [`Self::freeze`] has consumed it.
#[derive(Clone, Debug, Default)]
pub struct ConfidentialFixtureRegistry {
    entries: BTreeMap<String, ResolvedFixture>,
}

impl ConfidentialFixtureRegistry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one manifest, validating and deriving it.
    ///
    /// Handle grammar, material class, manifest closure, exhaustive and
    /// disjoint roles, positive semantic amounts, the explicit protocol
    /// asset on every protocol member, bounded counters, opening
    /// well-formedness where the contract carries them, and the digest,
    /// in that order. Nothing is repaired, defaulted, or retried.
    ///
    /// # Errors
    ///
    /// [`RegistrationRefusal`] at the first clause the manifest fails.
    pub fn register(
        &mut self,
        manifest: ConfidentialFixtureManifest,
    ) -> Result<(), RegistrationRefusal> {
        self.register_with_source(manifest, &TaggedHashDerivation)
    }

    /// Registers one manifest, deriving it from `source`.
    ///
    /// The seam [`DerivationSource`] documents, and the reason
    /// [`Self::register`] is a one-line call rather than the whole
    /// method: everything below is the registration, and the only thing
    /// the public entry point decides is that the source is the
    /// deterministic one.
    ///
    /// # Errors
    ///
    /// [`RegistrationRefusal`] at the first clause the manifest fails.
    pub(crate) fn register_with_source(
        &mut self,
        manifest: ConfidentialFixtureManifest,
        source: &dyn DerivationSource,
    ) -> Result<(), RegistrationRefusal> {
        check_handle_grammar(&manifest.handle)
            .map_err(|defect| RegistrationRefusal::HandleGrammar { defect })?;
        if self.entries.contains_key(manifest.handle.as_str()) {
            return Err(RegistrationRefusal::DuplicateFixtureHandle);
        }
        if manifest.material_class != PublicDisposableTestMaterial::EXPECTED {
            return Err(RegistrationRefusal::MaterialClassNotAdmitted);
        }
        if manifest.outputs.len() < 2 {
            return Err(RegistrationRefusal::OutputSetTooSmall {
                found: manifest.outputs.len(),
            });
        }
        let balancing = manifest
            .outputs
            .iter()
            .filter(|output| output.role == FixtureOutputRole::Balancing)
            .count();
        if balancing != 1 {
            return Err(RegistrationRefusal::BalancingRoleNotUnique { found: balancing });
        }
        for (index, output) in manifest.outputs.iter().enumerate() {
            if output.semantic_amount == 0 {
                return Err(RegistrationRefusal::AmountNotPositive { output: index });
            }
            if !is_semantic_amount(output.semantic_amount) {
                return Err(RegistrationRefusal::AmountNotSemantic { output: index });
            }
            if output.output_program.is_empty() {
                return Err(RegistrationRefusal::OutputProgramEmpty { output: index });
            }
        }
        if manifest.retry_limit > MAX_PARITY_COUNTER {
            return Err(RegistrationRefusal::RetryLimitAboveBound {
                stated: manifest.retry_limit,
            });
        }
        let input_blinder_sum = commitment::read_scalar(&manifest.input_blinder_sum)
            .map_err(|_| RegistrationRefusal::InputBlinderSumInvalid)?;

        let openings = match manifest.profiles.reproducibility_contract {
            ReproducibilityContract::ByteIdentity => {
                let (parity_counter, openings) =
                    search_parity(source, &manifest, &input_blinder_sum)
                        .map_err(|refusal| RegistrationRefusal::Derivation { refusal })?;
                FixtureOpenings::Derived {
                    parity_counter,
                    openings,
                }
            }
            ReproducibilityContract::RecordedRandomness => FixtureOpenings::RunProduced,
        };

        let digest = ConfidentialFixtureDigest::new(tagged_hash(
            FIXTURE_DIGEST_TAG,
            &digest_transcript(&manifest, &openings),
        ));
        let resolved = ResolvedFixture {
            handle: manifest.handle.clone(),
            digest,
            material_class: manifest.material_class,
            derivation_profile: manifest.derivation_profile,
            profiles: manifest.profiles,
            retry_limit: manifest.retry_limit,
            explicit_asset: manifest.explicit_asset,
            input_blinder_sum: manifest.input_blinder_sum,
            outputs: manifest.outputs,
            openings,
        };
        self.entries
            .insert(resolved.handle.as_str().to_owned(), resolved);
        Ok(())
    }

    /// Freezes the registry.
    ///
    /// A type transition and not a flag: this value is consumed, so a
    /// late write is not a rule that could be broken.
    #[must_use]
    pub fn freeze(self) -> FrozenConfidentialFixtureRegistry {
        FrozenConfidentialFixtureRegistry {
            entries: self.entries,
        }
    }
}

/// The registry, after the freeze.
///
/// The only thing with lookup authority, and the only thing that
/// supplies openings.
#[derive(Clone, Debug)]
pub struct FrozenConfidentialFixtureRegistry {
    entries: BTreeMap<String, ResolvedFixture>,
}

impl FrozenConfidentialFixtureRegistry {
    /// Resolves one handle at one digest.
    ///
    /// Both are required and neither is repaired: an unknown handle and
    /// a drifted digest are two different construction refusals, and
    /// nothing downstream may default or retry a lookup.
    ///
    /// # Errors
    ///
    /// [`FixtureResolutionRefused`] where the handle is unregistered or
    /// the registered digest differs.
    pub fn resolve(
        &self,
        handle: &ConfidentialFixtureHandle,
        digest: &ConfidentialFixtureDigest,
    ) -> Result<&ResolvedFixture, FixtureResolutionRefused> {
        let resolved = self
            .entries
            .get(handle.as_str())
            .ok_or(FixtureResolutionRefused::UnknownHandle)?;
        if resolved.digest != *digest {
            return Err(FixtureResolutionRefused::DigestMismatch);
        }
        Ok(resolved)
    }

    /// The digest one handle was registered under.
    ///
    /// What a ceremony plan reads to learn what it registered, so that
    /// the request it writes carries the registry's own digest rather
    /// than a recomputation beside it. It is not a lookup bypass: the
    /// fixture itself still resolves only against both.
    #[must_use]
    pub fn registered_digest(
        &self,
        handle: &ConfidentialFixtureHandle,
    ) -> Option<&ConfidentialFixtureDigest> {
        self.entries
            .get(handle.as_str())
            .map(|resolved| &resolved.digest)
    }

    /// Every registered handle, in the registry's own order.
    #[must_use]
    pub fn handles(&self) -> Vec<ConfidentialFixtureHandle> {
        self.entries
            .values()
            .map(|resolved| resolved.handle.clone())
            .collect()
    }

    /// One diagnostic for a lookup that did not resolve.
    #[must_use]
    pub fn diagnose(
        &self,
        handle: &ConfidentialFixtureHandle,
        refused: FixtureResolutionRefused,
    ) -> FixtureDiagnostic {
        let kind = match refused {
            FixtureResolutionRefused::UnknownHandle => FixtureDiagnosticKind::HandleUnknown,
            FixtureResolutionRefused::DigestMismatch => FixtureDiagnosticKind::DigestDrifted,
        };
        FixtureDiagnostic::new(
            handle.clone(),
            self.registered_digest(handle).copied(),
            None,
            None,
            kind,
        )
    }
}

impl ConfidentialFixtureResolution for FrozenConfidentialFixtureRegistry {
    fn resolve(
        &self,
        handle: &ConfidentialFixtureHandle,
        digest: &ConfidentialFixtureDigest,
    ) -> Result<(), FixtureResolutionRefused> {
        Self::resolve(self, handle, digest).map(|_| ())
    }
}

/// The first slice's case identity.
///
/// The spelling encodes no amount, no unit, no asset, no opening, no
/// derivation value, no digest fragment, no retry result, and no
/// transaction identity.
pub const PREDECESSOR_HANDLE: &str = "ctf-v1/predecessor-dual-parity";

/// The first slice's handle.
#[must_use]
pub fn predecessor_handle() -> ConfidentialFixtureHandle {
    ConfidentialFixtureHandle::new(PREDECESSOR_HANDLE.to_owned())
}

/// The scalar width every derived value carries.
///
/// Stated once here so a reader of this module does not have to reach
/// into the oracle for it; the value itself is the oracle's.
pub const DERIVED_SCALAR_BYTES: usize = SCALAR_BYTES;
