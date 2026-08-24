//! The confidential funding exchange: negotiation, strict framing, and
//! readback binding.
//!
//! # What this module is, and what it is not
//!
//! It is the harness's half of one wire arm. It decides what may be
//! sent, refuses what may not, and derives every output fact of an
//! answer from decoded mined bytes rather than from the answer's own
//! echo of the question.
//!
//! It is not a materializer, not a fixture registry, and not a validated
//! evidence record. The materializer lives with the executor that
//! advertised it; the registry that resolves a handle is reached through
//! [`ConfidentialFixtureResolution`] and is implemented elsewhere; and
//! the validated record that a report cites is assembled by the wave
//! that owns evidence. What is here is the boundary between them.
//!
//! # Every refusal is a construction or protocol refusal
//!
//! No variant of [`ConfidentialFundingRefusal`] is a target verdict, and
//! none may be read as one. A pre-target refusal is never a verdict, a
//! fixture-lookup failure is not evidence about a chain, and a refusal
//! record may carry no funded observation at all — the last of those has
//! its own variant, because a refusal that carried an asset, an
//! outpoint, an identity, or raw mined bytes would be a verdict wearing
//! a refusal's name.
//!
//! # Nothing here reaches a target
//!
//! The three collaborators this module needs — a decoder for mined
//! bytes, an independent inclusion observation, and a rangeproof
//! verifier — are traits, implemented outside this package. That is the
//! same construction the transaction-side capabilities already use, and
//! it is what keeps the package boundary intact: the decoder lives where
//! transaction bytes are understood, which is a package this one has no
//! library edge to.

use std::collections::BTreeSet;

use serde_json::{Map, Value};

use target_elements::ReproducibilityContract;

use crate::protocol::{
    ConfidentialFixtureDigest, ConfidentialFixtureHandle, ConfidentialFundedOutput,
    ConfidentialFundingAdvertisement, ConfidentialFundingBinding, ConfidentialFundingProfiles,
    ExecutorCapability, ExecutorHandshake, FundingCustodyProfile, FundingMaterializerProfile,
    FundingRepresentationProfile, NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse,
    ObservedOutcomeLayer, OperationStepKind, TargetConfidentialFundingSubject, WireOutpoint,
};

/// The serialized width of a confidential field, prefix included.
///
/// Read from the reviewed target contract rather than written here
/// twice: the value and nonce fields share the width, and the contract
/// is the single source of it.
fn committed_field_width() -> usize {
    target_elements::reviewed_confidential_review_facts()
        .value()
        .committed_width()
}

/// Which record a strict-framing refusal is about.
///
/// A closed vocabulary rather than a free string, so that a diagnostic
/// naming a record cannot name one the protocol does not define.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfidentialWireRecord {
    /// The confidential funding subject.
    Subject,
    /// One destination of that subject.
    Destination,
    /// The subject's binding.
    Binding,
    /// The binding's profiles.
    Profiles,
}

impl std::fmt::Display for ConfidentialWireRecord {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Subject => "confidential funding subject",
            Self::Destination => "confidential funding destination",
            Self::Binding => "confidential funding binding",
            Self::Profiles => "confidential funding profiles",
        };
        formatter.write_str(text)
    }
}

/// Which of the four profiles a refusal is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FundingProfileKind {
    /// The representation profile.
    Representation,
    /// The custody profile.
    Custody,
    /// The materializer profile.
    Materializer,
    /// The reproducibility contract.
    ReproducibilityContract,
}

impl std::fmt::Display for FundingProfileKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Representation => "representation",
            Self::Custody => "custody",
            Self::Materializer => "materializer",
            Self::ReproducibilityContract => "reproducibility contract",
        };
        formatter.write_str(text)
    }
}

/// Which readback projection differed.
///
/// The specific refusals take precedence: a missing rangeproof, an
/// inadmissible commitment encoding, a returned scalar, and the rest are
/// their own variants, and this vocabulary covers exactly the
/// projections that have no more specific name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MinedReadbackFact {
    /// No readback was reported at all.
    ReadbackAbsent,
    /// The identity recomputed from the decoded bytes.
    TransactionIdentity,
    /// The witness identity recomputed from the decoded bytes.
    WitnessTransactionIdentity,
    /// The transaction an outpoint names.
    OutpointTransaction,
    /// The output-witness entry an output names.
    OutputWitnessIndex,
    /// The value commitment in the decoded output.
    ValueCommitment,
    /// The nonce field in the decoded output.
    NonceField,
    /// The script the decoded output pays to.
    OutputScript,
    /// The rangeproof bytes in the decoded output.
    RangeproofBytes,
    /// The surjection-proof bytes in the decoded output.
    SurjectionProofBytes,
}

impl std::fmt::Display for MinedReadbackFact {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::ReadbackAbsent => "the mined readback",
            Self::TransactionIdentity => "the transaction identity",
            Self::WitnessTransactionIdentity => "the witness transaction identity",
            Self::OutpointTransaction => "the outpoint's transaction",
            Self::OutputWitnessIndex => "the output-witness index",
            Self::ValueCommitment => "the value commitment",
            Self::NonceField => "the nonce field",
            Self::OutputScript => "the output script",
            Self::RangeproofBytes => "the rangeproof bytes",
            Self::SurjectionProofBytes => "the surjection-proof bytes",
        };
        formatter.write_str(text)
    }
}

/// Why one confidential funding exchange was refused.
///
/// Closed, with no catch-all. Every variant is a construction refusal or
/// a protocol refusal, and never a target verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfidentialFundingRefusal {
    /// The peer does not speak the required revision.
    ProtocolSchemaUnsupported {
        /// What this harness speaks.
        required: u32,
        /// What the peer offered.
        offered: u32,
    },
    /// The confidential funding capability is not advertised.
    ConfidentialFundingCapabilityAbsent,
    /// The capability and the advertisement record contradict each
    /// other.
    CapabilityAdvertisementDisagrees,
    /// The exact hybrid representation is not advertised.
    HybridRepresentationUnsupported,
    /// A representation tag no arm is assigned to.
    UnknownRepresentationTag {
        /// The tag as it was written.
        tag: String,
    },
    /// A profile unknown or unadvertised.
    FundingProfileUnsupported {
        /// Which profile.
        kind: FundingProfileKind,
        /// The name as it was written.
        name: String,
    },
    /// Strict framing failed on a member.
    UnknownWireMember {
        /// Which record.
        record: ConfidentialWireRecord,
        /// The member at fault: present and undeclared, or declared and
        /// absent.
        member: String,
    },
    /// Registry lookup found no such handle.
    UnknownFixtureHandle {
        /// The handle that was looked up.
        handle: ConfidentialFixtureHandle,
    },
    /// The registered digest differs from the requested one.
    FixtureDigestMismatch {
        /// The handle whose digest drifted.
        handle: ConfidentialFixtureHandle,
    },
    /// A confidential request with no destination.
    DestinationSetEmpty,
    /// The materializer refused, with the non-verdict layer it refused
    /// at as its typed cause.
    DeterministicMaterializationRefused {
        /// The layer the executor reported.
        cause: ObservedOutcomeLayer,
    },
    /// The answer belongs to another arm, explicit fallback included.
    ResponseArmMismatch {
        /// The arm that was asked for.
        requested: OperationStepKind,
        /// The arm that answered.
        returned: OperationStepKind,
    },
    /// The handle or digest differs from the request.
    ResponseFixtureBindingMismatch,
    /// The profile binding differs from the request.
    ResponseProfileBindingMismatch,
    /// The answer reports a different number of outputs.
    OutputCountMismatch {
        /// How many destinations were requested.
        requested: usize,
        /// How many outputs were observed.
        observed: usize,
    },
    /// A mined output pays a program the request did not state.
    OutputProgramMismatch {
        /// Which output.
        index: usize,
    },
    /// A mined output carries another explicit asset.
    ExplicitAssetMismatch {
        /// Which output.
        index: usize,
    },
    /// An asset commitment came back where an explicit asset was asked
    /// for.
    ConfidentialAssetReturned {
        /// Which output.
        index: usize,
    },
    /// A scalar came back where a commitment was asked for.
    ExplicitValueReturned {
        /// Which output.
        index: usize,
    },
    /// A commitment is not an admitted prefix or not a point.
    CommitmentEncodingInvalid {
        /// Which output.
        index: usize,
    },
    /// A nonce field is absent or not admitted by the reviewed encoding.
    NonceEncodingInvalid {
        /// Which output.
        index: usize,
    },
    /// A rangeproof field is empty.
    RangeproofMissing {
        /// Which output.
        index: usize,
    },
    /// A rangeproof did not verify.
    RangeproofInvalid {
        /// Which output.
        index: usize,
    },
    /// A surjection proof carries bytes where the representation
    /// requires none.
    SurjectionProofUnexpected {
        /// Which output.
        index: usize,
    },
    /// One coin was reported twice.
    DuplicateFundedOutpoint {
        /// The outpoint reported twice.
        outpoint: WireOutpoint,
    },
    /// The readback bytes do not decode.
    RawTransactionDecodeFailed,
    /// The named block does not contain the transaction.
    MinedInclusionUnverified,
    /// A readback projection differs.
    MinedReadbackMismatch {
        /// Which projection.
        fact: MinedReadbackFact,
        /// Which output, where the projection belongs to one.
        index: Option<usize>,
    },
    /// A refusal carried an asset, an outpoint, an identity, or raw
    /// bytes.
    RefusalCarriesFundedObservation,
}

impl std::fmt::Display for ConfidentialFundingRefusal {
    /// One arm per variant, and the match is exhaustive.
    ///
    /// Long for that reason rather than by accident: a variant added to
    /// the vocabulary has no rendering until one is written here, and
    /// the compiler is what says so. Splitting the match into shorter
    /// halves would need a catch-all arm in each, which is exactly the
    /// fallback a closed refusal vocabulary exists to avoid.
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProtocolSchemaUnsupported { required, offered } => write!(
                formatter,
                "the peer speaks protocol revision {offered} and this harness speaks {required}",
            ),
            Self::ConfidentialFundingCapabilityAbsent => {
                formatter.write_str("the executor advertises no confidential funding capability")
            }
            Self::CapabilityAdvertisementDisagrees => formatter.write_str(
                "the confidential funding capability and its advertisement contradict each other",
            ),
            Self::HybridRepresentationUnsupported => formatter
                .write_str("the executor does not advertise the exact hybrid representation"),
            Self::UnknownRepresentationTag { tag } => {
                write!(formatter, "no arm is assigned the representation tag {tag}")
            }
            Self::FundingProfileUnsupported { kind, name } => write!(
                formatter,
                "the {kind} profile {name} is unknown or unadvertised",
            ),
            Self::UnknownWireMember { record, member } => {
                write!(formatter, "the {record} does not frame the member {member}")
            }
            Self::UnknownFixtureHandle { handle } => {
                write!(formatter, "no fixture is registered under {handle}")
            }
            Self::FixtureDigestMismatch { handle } => write!(
                formatter,
                "the fixture registered under {handle} carries another digest",
            ),
            Self::DestinationSetEmpty => {
                formatter.write_str("the confidential request states no destination")
            }
            Self::DeterministicMaterializationRefused { cause } => write!(
                formatter,
                "the executor refused deterministic materialization: {cause}",
            ),
            Self::ResponseArmMismatch {
                requested,
                returned,
            } => write!(
                formatter,
                "a {requested} step was answered by a {returned} step",
            ),
            Self::ResponseFixtureBindingMismatch => {
                formatter.write_str("the recorded fixture binding differs from the request")
            }
            Self::ResponseProfileBindingMismatch => {
                formatter.write_str("the recorded profile binding differs from the request")
            }
            Self::OutputCountMismatch {
                requested,
                observed,
            } => write!(
                formatter,
                "{requested} destinations were requested and {observed} outputs observed",
            ),
            Self::OutputProgramMismatch { index } => {
                write!(formatter, "output {index} pays an unrequested program")
            }
            Self::ExplicitAssetMismatch { index } => {
                write!(formatter, "output {index} carries another explicit asset")
            }
            Self::ConfidentialAssetReturned { index } => {
                write!(formatter, "output {index} returned an asset commitment")
            }
            Self::ExplicitValueReturned { index } => {
                write!(formatter, "output {index} returned an explicit value")
            }
            Self::CommitmentEncodingInvalid { index } => write!(
                formatter,
                "output {index} carries no admitted value commitment",
            ),
            Self::NonceEncodingInvalid { index } => {
                write!(formatter, "output {index} carries no admitted nonce field")
            }
            Self::RangeproofMissing { index } => {
                write!(formatter, "output {index} carries no rangeproof")
            }
            Self::RangeproofInvalid { index } => {
                write!(
                    formatter,
                    "output {index} carries a rangeproof that does not verify"
                )
            }
            Self::SurjectionProofUnexpected { index } => {
                write!(formatter, "output {index} carries a surjection proof")
            }
            Self::DuplicateFundedOutpoint { outpoint } => write!(
                formatter,
                "the coin {}:{} was reported twice",
                outpoint.txid, outpoint.vout,
            ),
            Self::RawTransactionDecodeFailed => {
                formatter.write_str("the readback bytes do not decode")
            }
            Self::MinedInclusionUnverified => {
                formatter.write_str("the named block does not contain the transaction")
            }
            Self::MinedReadbackMismatch { fact, index } => match index {
                Some(index) => write!(formatter, "{fact} of output {index} differs"),
                None => write!(formatter, "{fact} differs"),
            },
            Self::RefusalCarriesFundedObservation => {
                formatter.write_str("a refusal carried a funded observation")
            }
        }
    }
}

/// What a strict-framing attempt produced.
///
/// Two outcomes rather than one, because they are two different
/// statements. A typed refusal names a fault the confidential
/// vocabulary holds a word for; a malformed record is the transport's
/// own framing failure, which this vocabulary deliberately does not
/// restate — a value of the wrong JSON type was never a confidential
/// question.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfidentialFramingRefusal {
    /// A typed wire refusal.
    Refused(ConfidentialFundingRefusal),
    /// The record is not a shape this protocol defines at all.
    Malformed,
}

impl std::fmt::Display for ConfidentialFramingRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Refused(refusal) => write!(formatter, "{refusal}"),
            Self::Malformed => {
                formatter.write_str("the record is not a shape this protocol defines")
            }
        }
    }
}

/// The members the confidential funding subject declares.
const SUBJECT_MEMBERS: [&str; 4] = ["issue_asset", "asset", "destinations", "binding"];

/// The members one destination declares.
const DESTINATION_MEMBERS: [&str; 1] = ["output_program"];

/// The members the binding declares.
const BINDING_MEMBERS: [&str; 3] = ["fixture_handle", "fixture_digest", "profiles"];

/// The members the profiles declare.
const PROFILES_MEMBERS: [&str; 4] = [
    "representation",
    "custody",
    "materializer",
    "reproducibility_contract",
];

/// Every member of a record is declared, and every declared member is
/// present.
///
/// Both directions, because both are strict-framing failures: an
/// undeclared member is a record this side does not define, and an
/// absent one is a record that is not the shape it claims to be. Neither
/// is ignored, defaulted, or repaired.
fn framed_members(
    record: ConfidentialWireRecord,
    object: &Map<String, Value>,
    declared: &[&str],
) -> Result<(), ConfidentialFundingRefusal> {
    for member in object.keys() {
        if !declared.contains(&member.as_str()) {
            return Err(ConfidentialFundingRefusal::UnknownWireMember {
                record,
                member: member.clone(),
            });
        }
    }
    for member in declared {
        if !object.contains_key(*member) {
            return Err(ConfidentialFundingRefusal::UnknownWireMember {
                record,
                member: (*member).to_owned(),
            });
        }
    }
    Ok(())
}

/// One tag, read as an assigned member of a closed vocabulary.
fn assigned_tag<'a>(object: &'a Map<String, Value>, member: &str) -> Option<&'a str> {
    object.get(member).and_then(Value::as_str)
}

/// The profiles frame, tag by tag.
///
/// The representation has its own refusal because it is the arm: a tag
/// no arm is assigned to is not a profile that happens to be unknown.
fn framed_profiles(object: &Map<String, Value>) -> Result<(), ConfidentialFundingRefusal> {
    framed_members(ConfidentialWireRecord::Profiles, object, &PROFILES_MEMBERS)?;
    let representation =
        assigned_tag(object, "representation").ok_or_else(|| unframed("representation"))?;
    if serde_json::from_value::<FundingRepresentationProfile>(Value::from(representation)).is_err()
    {
        return Err(ConfidentialFundingRefusal::UnknownRepresentationTag {
            tag: representation.to_owned(),
        });
    }

    let custody = assigned_tag(object, "custody").ok_or_else(|| unframed("custody"))?;
    if serde_json::from_value::<FundingCustodyProfile>(Value::from(custody)).is_err() {
        return Err(ConfidentialFundingRefusal::FundingProfileUnsupported {
            kind: FundingProfileKind::Custody,
            name: custody.to_owned(),
        });
    }

    let materializer =
        assigned_tag(object, "materializer").ok_or_else(|| unframed("materializer"))?;
    if serde_json::from_value::<FundingMaterializerProfile>(Value::from(materializer)).is_err() {
        return Err(ConfidentialFundingRefusal::FundingProfileUnsupported {
            kind: FundingProfileKind::Materializer,
            name: materializer.to_owned(),
        });
    }

    let contract = assigned_tag(object, "reproducibility_contract")
        .ok_or_else(|| unframed("reproducibility_contract"))?;
    if ReproducibilityContract::from_code(contract).is_none() {
        return Err(ConfidentialFundingRefusal::FundingProfileUnsupported {
            kind: FundingProfileKind::ReproducibilityContract,
            name: contract.to_owned(),
        });
    }
    Ok(())
}

/// One member of the profiles record that is not framed as a tag.
fn unframed(member: &str) -> ConfidentialFundingRefusal {
    ConfidentialFundingRefusal::UnknownWireMember {
        record: ConfidentialWireRecord::Profiles,
        member: member.to_owned(),
    }
}

/// Reads one confidential funding subject under strict framing.
///
/// # An untagged record under this revision is unknown, not explicit
///
/// The step kind decides which arm a subject is read as, exactly as the
/// reviewed adapter reads it. A record carrying the explicit arm's
/// members is therefore refused HERE, naming the member the confidential
/// subject does not declare, rather than parsing as the explicit arm
/// because it happened to fit. Nothing falls back, and nothing is
/// ignored.
///
/// # Errors
///
/// [`ConfidentialFramingRefusal`] where a member is undeclared or
/// absent, a tag is unassigned, a profile is unknown, or the record is
/// not a shape this protocol defines.
pub fn frame_confidential_subject(
    record: &Map<String, Value>,
) -> Result<TargetConfidentialFundingSubject, ConfidentialFramingRefusal> {
    framed_members(ConfidentialWireRecord::Subject, record, &SUBJECT_MEMBERS)
        .map_err(ConfidentialFramingRefusal::Refused)?;

    let destinations = record
        .get("destinations")
        .and_then(Value::as_array)
        .ok_or(ConfidentialFramingRefusal::Malformed)?;
    for destination in destinations {
        let object = destination
            .as_object()
            .ok_or(ConfidentialFramingRefusal::Malformed)?;
        framed_members(
            ConfidentialWireRecord::Destination,
            object,
            &DESTINATION_MEMBERS,
        )
        .map_err(ConfidentialFramingRefusal::Refused)?;
    }

    let binding = record
        .get("binding")
        .and_then(Value::as_object)
        .ok_or(ConfidentialFramingRefusal::Malformed)?;
    framed_members(ConfidentialWireRecord::Binding, binding, &BINDING_MEMBERS)
        .map_err(ConfidentialFramingRefusal::Refused)?;
    let profiles = binding
        .get("profiles")
        .and_then(Value::as_object)
        .ok_or(ConfidentialFramingRefusal::Malformed)?;
    framed_profiles(profiles).map_err(ConfidentialFramingRefusal::Refused)?;

    serde_json::from_value(Value::Object(record.clone()))
        .map_err(|_| ConfidentialFramingRefusal::Malformed)
}

/// What the executor advertised, checked against what it claimed.
///
/// # The two halves may not disagree
///
/// A capability without an advertisement is an interface with no stated
/// selections; an advertisement without a capability is a set of
/// selections nothing offers. Either one is refused rather than resolved
/// in whichever direction is convenient, because resolving it would mean
/// choosing what the peer meant.
///
/// # Errors
///
/// [`ConfidentialFundingRefusal`] where the revision, the capability, or
/// the advertisement does not admit a confidential exchange.
pub fn negotiate_confidential_funding(
    handshake: &ExecutorHandshake,
) -> Result<&ConfidentialFundingAdvertisement, ConfidentialFundingRefusal> {
    if handshake.protocol_schema != NATIVE_PROTOCOL_SCHEMA {
        return Err(ConfidentialFundingRefusal::ProtocolSchemaUnsupported {
            required: NATIVE_PROTOCOL_SCHEMA,
            offered: handshake.protocol_schema,
        });
    }
    let claims = handshake
        .capabilities
        .contains(&ExecutorCapability::ConfidentialValueTestFunding);
    match (claims, handshake.confidential_funding.as_ref()) {
        (true, Some(advertisement)) => Ok(advertisement),
        (false, None) => Err(ConfidentialFundingRefusal::ConfidentialFundingCapabilityAbsent),
        (true, None) | (false, Some(_)) => {
            Err(ConfidentialFundingRefusal::CapabilityAdvertisementDisagrees)
        }
    }
}

/// Every selected profile appears in what the executor advertised.
///
/// # Advertisement constrains, and never chooses
///
/// The selection is the ceremony plan's. This check is what makes an
/// unsupported selection a refusal BEFORE the request is written rather
/// than a downgrade after it: nothing here picks a profile, and nothing
/// here falls back to one the executor would have preferred.
///
/// # Errors
///
/// [`ConfidentialFundingRefusal`] where a selected profile is not
/// advertised.
pub fn check_selected_profiles(
    advertisement: &ConfidentialFundingAdvertisement,
    profiles: &ConfidentialFundingProfiles,
) -> Result<(), ConfidentialFundingRefusal> {
    if !advertisement
        .representation_profiles
        .contains(&profiles.representation)
    {
        return Err(ConfidentialFundingRefusal::HybridRepresentationUnsupported);
    }
    if !advertisement.custody_profiles.contains(&profiles.custody) {
        return Err(ConfidentialFundingRefusal::FundingProfileUnsupported {
            kind: FundingProfileKind::Custody,
            name: wire_tag(&profiles.custody),
        });
    }
    if !advertisement
        .materializer_profiles
        .contains(&profiles.materializer)
    {
        return Err(ConfidentialFundingRefusal::FundingProfileUnsupported {
            kind: FundingProfileKind::Materializer,
            name: wire_tag(&profiles.materializer),
        });
    }
    if !advertisement
        .reproducibility_contracts
        .contains(&profiles.reproducibility_contract)
    {
        return Err(ConfidentialFundingRefusal::FundingProfileUnsupported {
            kind: FundingProfileKind::ReproducibilityContract,
            name: profiles.reproducibility_contract.code().to_owned(),
        });
    }
    Ok(())
}

/// One profile's own wire tag.
///
/// Read through the serialization the wire uses rather than rendered a
/// second way, so a refusal names a profile in the same word the request
/// carried it in `(´[PLAN-rule:guide12-exec:typed-source]´)`.
pub(crate) fn wire_tag<T: serde::Serialize>(profile: &T) -> String {
    serde_json::to_value(profile)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// Why a fixture lookup did not resolve.
///
/// The registry's two answers, and neither of them is a target verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FixtureResolutionRefused {
    /// No fixture is registered under the handle.
    UnknownHandle,
    /// A fixture is registered and carries another digest.
    DigestMismatch,
}

/// The only thing with lookup authority.
///
/// No environment value, no wallet, no request member other than the
/// handle and the digest, and no response echo may resolve a fixture.
/// The registry that implements this is frozen before any lookup; what
/// this package needs of it is the answer, not its shape.
pub trait ConfidentialFixtureResolution {
    /// Resolves one handle at one digest.
    ///
    /// # Errors
    ///
    /// [`FixtureResolutionRefused`] where the handle is unregistered or
    /// the registered digest differs.
    fn resolve(
        &self,
        handle: &ConfidentialFixtureHandle,
        digest: &ConfidentialFixtureDigest,
    ) -> Result<(), FixtureResolutionRefused>;
}

/// Resolves a request's binding before any cryptographic work.
///
/// An unknown handle and a drifted digest are construction refusals at
/// this point, and nothing downstream may repair, default, or retry a
/// lookup.
///
/// # Errors
///
/// [`ConfidentialFundingRefusal`] where the lookup did not resolve.
pub fn resolve_binding(
    registry: &dyn ConfidentialFixtureResolution,
    binding: &ConfidentialFundingBinding,
) -> Result<(), ConfidentialFundingRefusal> {
    registry
        .resolve(&binding.fixture_handle, &binding.fixture_digest)
        .map_err(|refused| match refused {
            FixtureResolutionRefused::UnknownHandle => {
                ConfidentialFundingRefusal::UnknownFixtureHandle {
                    handle: binding.fixture_handle.clone(),
                }
            }
            FixtureResolutionRefused::DigestMismatch => {
                ConfidentialFundingRefusal::FixtureDigestMismatch {
                    handle: binding.fixture_handle.clone(),
                }
            }
        })
}

/// Everything that must hold before a confidential request is written.
///
/// The order is the fail-closed one: the revision, then the capability
/// and its advertisement, then the selection against that
/// advertisement, then the request's own shape, and only then the
/// registry lookup. A request that reaches the executor has passed all
/// five.
///
/// # Errors
///
/// [`ConfidentialFundingRefusal`] at the first check that does not hold.
pub fn check_before_sending(
    handshake: &ExecutorHandshake,
    subject: &TargetConfidentialFundingSubject,
    registry: &dyn ConfidentialFixtureResolution,
) -> Result<(), ConfidentialFundingRefusal> {
    let advertisement = negotiate_confidential_funding(handshake)?;
    check_selected_profiles(advertisement, &subject.binding.profiles)?;
    if subject.destinations.is_empty() {
        return Err(ConfidentialFundingRefusal::DestinationSetEmpty);
    }
    resolve_binding(registry, &subject.binding)
}

/// One decoded output field that is either explicit or committed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodedAssetField {
    /// An explicit asset, in the target's own spelling.
    Explicit(String),
    /// An asset commitment.
    Commitment(Vec<u8>),
}

/// One decoded value field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodedValueField {
    /// An explicit amount.
    Explicit(u64),
    /// A value commitment, prefix included.
    Commitment(Vec<u8>),
}

/// One output, as the decoder read it out of the mined bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedFundingOutput {
    /// The decoded asset field.
    pub asset: DecodedAssetField,
    /// The decoded value field.
    pub value: DecodedValueField,
    /// The decoded nonce field, empty where the field is null.
    pub nonce: Vec<u8>,
    /// The decoded output program.
    pub program: Vec<u8>,
    /// The decoded surjection proof.
    pub surjection_proof: Vec<u8>,
    /// The decoded rangeproof.
    pub rangeproof: Vec<u8>,
}

/// One transaction, as the decoder read it out of the mined bytes.
///
/// The two identities are RECOMPUTED from the bytes rather than copied
/// from the answer. That is the whole reason this record exists: a
/// reported identity is a claim, and a recomputed one is a reading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedFundingTransaction {
    /// The identity recomputed from the decoded bytes.
    pub transaction_id: String,
    /// The witness identity recomputed from the decoded bytes.
    pub witness_transaction_id: String,
    /// The decoded outputs, in the order the bytes carry them.
    pub outputs: Vec<DecodedFundingOutput>,
}

/// The decoder refused the bytes.
///
/// A marker rather than a cause. Why a byte string failed to decode
/// belongs to the package that understands transaction bytes, and
/// restating its vocabulary here would be a second spelling of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadbackDecodeRefused;

/// Reads mined bytes back into fields.
///
/// Implemented outside this package, because transaction bytes are
/// understood in a package this one has no library edge to.
pub trait ConfidentialReadbackDecoder {
    /// Decodes one mined transaction.
    ///
    /// # Errors
    ///
    /// [`ReadbackDecodeRefused`] where the bytes are not a transaction.
    fn decode(&self, raw: &[u8]) -> Result<DecodedFundingTransaction, ReadbackDecodeRefused>;
}

/// States whether a named block holds a named transaction.
///
/// An independent observation of the target, and never a projection of
/// the answer being checked.
pub trait MinedInclusionOracle {
    /// Whether the block at that height holds that transaction.
    fn contains(&self, block_hash: &str, block_height: u32, transaction_id: &str) -> bool;
}

/// Verifies one rangeproof against what it must bind to.
///
/// The proof binds this output's commitment, the unblinded generator of
/// the explicit asset, and this output's program. A verifier that
/// checked fewer of those would accept a proof built for some other
/// output.
pub trait RangeproofVerifier {
    /// Whether the proof verifies against exactly those three things.
    fn verifies(
        &self,
        proof: &[u8],
        value_commitment: &[u8],
        explicit_asset: &str,
        output_program: &[u8],
    ) -> bool;
}

/// The three collaborators the binding needs.
pub struct ConfidentialFundingOracles<'a> {
    /// Reads mined bytes back into fields.
    pub decoder: &'a dyn ConfidentialReadbackDecoder,
    /// States what the chain holds.
    pub inclusion: &'a dyn MinedInclusionOracle,
    /// Verifies rangeproofs.
    pub rangeproofs: &'a dyn RangeproofVerifier,
}

/// What was asked, and what the record under validation claims it was
/// asked under.
///
/// The two are separate on purpose. Binding the arm, the handle, the
/// digest, and the profiles is a comparison between what the caller
/// asked for and what the caller sent; every OUTPUT fact is a comparison
/// between the record and the chain. Collapsing the two would let an
/// echo stand in for a reading.
pub struct ConfidentialFundingExchange<'a> {
    /// The subject that was sent.
    pub requested: &'a TargetConfidentialFundingSubject,
    /// The binding the record under validation claims.
    pub recorded: &'a ConfidentialFundingBinding,
    /// What the executor advertised.
    pub advertisement: &'a ConfidentialFundingAdvertisement,
}

/// One output fact set, derived from the decoded bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadbackDerivedOutput {
    /// The coin, with the transaction identity recomputed.
    pub outpoint: WireOutpoint,
    /// The explicit protocol asset the decoded output carries.
    pub explicit_asset: String,
    /// The value commitment the decoded output carries.
    pub value_commitment: Vec<u8>,
    /// The nonce field the decoded output carries.
    pub nonce: Vec<u8>,
    /// The program the decoded output pays.
    pub output_program: Vec<u8>,
    /// The rangeproof the decoded output carries.
    pub rangeproof: Vec<u8>,
}

/// What the chain holds, derived rather than echoed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadbackDerivedFunding {
    /// The recomputed transaction identity.
    pub transaction_id: String,
    /// The recomputed witness transaction identity.
    pub witness_transaction_id: String,
    /// The block the target says holds it, verified independently.
    pub block_hash: String,
    /// That block's height.
    pub block_height: u32,
    /// The outputs, in fixture order.
    pub outputs: Vec<ReadbackDerivedOutput>,
}

/// What one confidential funding answer observed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfidentialFundingObservation {
    /// The target reached a verdict and created nothing.
    ///
    /// An ordinary negative answer rather than a contradiction: the
    /// whole point of the step is to record what the target did.
    NoFundedObservation,
    /// The target accepted, and these facts are readback-derived.
    Funded(Box<ReadbackDerivedFunding>),
}

/// Whether a response carries any funded observation at all.
const fn carries_funded_observation(response: &NativeOperationResponse) -> bool {
    !response.confidential_funded_outputs.is_empty()
        || response.mined_readback.is_some()
        || !response.funded_outputs.is_empty()
        || response.issued_asset.is_some()
        || response.accepted_txid.is_some()
}

/// The arm an answer belongs to, read from the answer rather than from
/// the question.
fn returned_arm(response: &NativeOperationResponse) -> OperationStepKind {
    if response.case.operation == OperationStepKind::FundConfidential
        && !response.funded_outputs.is_empty()
    {
        // The explicit fallback: a confidential question answered with
        // the explicit arm's outputs.
        return OperationStepKind::Fund;
    }
    response.case.operation
}

/// The binding the record claims equals the binding that was sent.
fn check_recorded_binding(
    exchange: &ConfidentialFundingExchange<'_>,
) -> Result<(), ConfidentialFundingRefusal> {
    let requested = &exchange.requested.binding;
    if exchange.recorded.fixture_handle != requested.fixture_handle
        || exchange.recorded.fixture_digest != requested.fixture_digest
    {
        return Err(ConfidentialFundingRefusal::ResponseFixtureBindingMismatch);
    }
    if exchange.recorded.profiles != requested.profiles {
        return Err(ConfidentialFundingRefusal::ResponseProfileBindingMismatch);
    }
    check_selected_profiles(exchange.advertisement, &exchange.recorded.profiles)
}

/// The explicit protocol asset the outputs must carry.
///
/// Stated by the request where an earlier step issued the asset, and
/// otherwise chosen by the target in this same step and reported by it.
/// There is no third source, and a step with neither has named no asset
/// at all.
fn expected_explicit_asset(
    subject: &TargetConfidentialFundingSubject,
    response: &NativeOperationResponse,
) -> Result<String, ConfidentialFundingRefusal> {
    subject
        .asset
        .clone()
        .or_else(|| response.issued_asset.clone())
        .ok_or(ConfidentialFundingRefusal::ExplicitAssetMismatch { index: 0 })
}

/// One output's readback comparison.
struct OutputBinding<'a> {
    index: usize,
    reported: &'a ConfidentialFundedOutput,
    decoded: &'a DecodedFundingOutput,
    expected_asset: &'a str,
    expected_program: &'a [u8],
    transaction_id: &'a str,
}

impl OutputBinding<'_> {
    /// A mismatch of one projection at this output.
    const fn differs(&self, fact: MinedReadbackFact) -> ConfidentialFundingRefusal {
        ConfidentialFundingRefusal::MinedReadbackMismatch {
            fact,
            index: Some(self.index),
        }
    }

    /// The asset and value forms are the exact hybrid tuple.
    fn check_forms(&self) -> Result<Vec<u8>, ConfidentialFundingRefusal> {
        let index = self.index;
        let asset = match &self.decoded.asset {
            DecodedAssetField::Commitment(_) => {
                return Err(ConfidentialFundingRefusal::ConfidentialAssetReturned { index });
            }
            DecodedAssetField::Explicit(asset) => asset,
        };
        if asset != self.expected_asset || self.reported.explicit_asset != *self.expected_asset {
            return Err(ConfidentialFundingRefusal::ExplicitAssetMismatch { index });
        }
        let commitment = match &self.decoded.value {
            DecodedValueField::Explicit(_) => {
                return Err(ConfidentialFundingRefusal::ExplicitValueReturned { index });
            }
            DecodedValueField::Commitment(commitment) => commitment.clone(),
        };
        if !admitted_commitment(&self.reported.value_commitment) {
            return Err(ConfidentialFundingRefusal::CommitmentEncodingInvalid { index });
        }
        if commitment != self.reported.value_commitment {
            return Err(self.differs(MinedReadbackFact::ValueCommitment));
        }
        Ok(commitment)
    }

    /// The program, the script rendering, and the nonce.
    fn check_fields(&self) -> Result<(), ConfidentialFundingRefusal> {
        let index = self.index;
        if self.decoded.program != self.expected_program {
            return Err(ConfidentialFundingRefusal::OutputProgramMismatch { index });
        }
        if !self
            .reported
            .script
            .eq_ignore_ascii_case(&render_hex(&self.decoded.program))
        {
            return Err(self.differs(MinedReadbackFact::OutputScript));
        }
        if !admitted_nonce(&self.decoded.nonce) || !admitted_nonce(&self.reported.nonce) {
            return Err(ConfidentialFundingRefusal::NonceEncodingInvalid { index });
        }
        if self.decoded.nonce != self.reported.nonce {
            return Err(self.differs(MinedReadbackFact::NonceField));
        }
        if self.reported.outpoint.txid != self.transaction_id {
            return Err(self.differs(MinedReadbackFact::OutpointTransaction));
        }
        if self.reported.output_witness_index != self.reported.outpoint.vout {
            return Err(self.differs(MinedReadbackFact::OutputWitnessIndex));
        }
        Ok(())
    }

    /// The proofs: present where required, empty where required, equal
    /// to what the chain holds, and bound to this output.
    fn check_proofs(
        &self,
        commitment: &[u8],
        verifier: &dyn RangeproofVerifier,
    ) -> Result<(), ConfidentialFundingRefusal> {
        let index = self.index;
        if !self.reported.surjection_proof.is_empty() || !self.decoded.surjection_proof.is_empty() {
            return Err(ConfidentialFundingRefusal::SurjectionProofUnexpected { index });
        }
        if self.reported.rangeproof.is_empty() || self.decoded.rangeproof.is_empty() {
            return Err(ConfidentialFundingRefusal::RangeproofMissing { index });
        }
        if self.decoded.rangeproof != self.reported.rangeproof {
            return Err(self.differs(MinedReadbackFact::RangeproofBytes));
        }
        if !verifier.verifies(
            &self.decoded.rangeproof,
            commitment,
            self.expected_asset,
            &self.decoded.program,
        ) {
            return Err(ConfidentialFundingRefusal::RangeproofInvalid { index });
        }
        Ok(())
    }

    /// Every fact of one output, derived from the decoded bytes.
    fn bind(
        &self,
        verifier: &dyn RangeproofVerifier,
    ) -> Result<ReadbackDerivedOutput, ConfidentialFundingRefusal> {
        let commitment = self.check_forms()?;
        self.check_fields()?;
        self.check_proofs(&commitment, verifier)?;
        Ok(ReadbackDerivedOutput {
            outpoint: WireOutpoint {
                txid: self.transaction_id.to_owned(),
                vout: self.reported.outpoint.vout,
            },
            explicit_asset: self.expected_asset.to_owned(),
            value_commitment: commitment,
            nonce: self.decoded.nonce.clone(),
            output_program: self.decoded.program.clone(),
            rangeproof: self.decoded.rangeproof.clone(),
        })
    }
}

/// Whether a serialized value commitment is one the reviewed encoding
/// admits.
///
/// Width and prefix from the reviewed target contract, and curve
/// membership from the first-party commitment oracle. Neither is
/// restated here: a literal written twice is a literal that can drift.
fn admitted_commitment(commitment: &[u8]) -> bool {
    if commitment.len() != committed_field_width() {
        return false;
    }
    let encoding = target_elements::reviewed_confidential_review_facts().value();
    let (even, odd) = encoding.committed_prefixes();
    if commitment[0] != even && commitment[0] != odd {
        return false;
    }
    crate::commitment_oracle::commitment::parse_commitment(commitment).is_ok()
}

/// Whether a serialized nonce field is one the reviewed encoding admits.
///
/// A null field is not admitted here, and that is the deliberate part: a
/// confidential output whose nonce field is absent carries no nonce for
/// anything to have been derived against.
fn admitted_nonce(nonce: &[u8]) -> bool {
    if nonce.len() != committed_field_width() {
        return false;
    }
    let encoding = target_elements::reviewed_confidential_review_facts().nonce();
    let (even, odd) = encoding.committed_prefixes();
    nonce[0] == even || nonce[0] == odd
}

/// Lowercase hexadecimal, which is the rendering the target reports
/// scripts in.
fn render_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

/// Binds one confidential funding answer to the chain and to the
/// request.
///
/// # No fact comes from an echo
///
/// The identities are recomputed from the decoded bytes, the inclusion
/// is an independent observation, and every output fact is read out of
/// the decoded output. What the response reports is compared against
/// those readings rather than believed, and the only things checked
/// against the REQUEST are the arm, the handle, the digest, and the
/// profiles — which is what the caller asked for, checked against what
/// the caller sent.
///
/// # Errors
///
/// [`ConfidentialFundingRefusal`] at the first condition that does not
/// hold.
pub fn bind_confidential_funding(
    exchange: &ConfidentialFundingExchange<'_>,
    response: &NativeOperationResponse,
    oracles: &ConfidentialFundingOracles<'_>,
) -> Result<ConfidentialFundingObservation, ConfidentialFundingRefusal> {
    let returned = returned_arm(response);
    if returned != OperationStepKind::FundConfidential {
        return Err(ConfidentialFundingRefusal::ResponseArmMismatch {
            requested: OperationStepKind::FundConfidential,
            returned,
        });
    }
    check_recorded_binding(exchange)?;

    if response.observed_layer != ObservedOutcomeLayer::Accepted {
        if carries_funded_observation(response) {
            return Err(ConfidentialFundingRefusal::RefusalCarriesFundedObservation);
        }
        return if response.observed_layer.is_target_verdict() {
            Ok(ConfidentialFundingObservation::NoFundedObservation)
        } else {
            Err(
                ConfidentialFundingRefusal::DeterministicMaterializationRefused {
                    cause: response.observed_layer,
                },
            )
        };
    }

    let readback = response.mined_readback.as_ref().ok_or(
        ConfidentialFundingRefusal::MinedReadbackMismatch {
            fact: MinedReadbackFact::ReadbackAbsent,
            index: None,
        },
    )?;
    let decoded = oracles
        .decoder
        .decode(&readback.raw_transaction)
        .map_err(|ReadbackDecodeRefused| ConfidentialFundingRefusal::RawTransactionDecodeFailed)?;
    if decoded.transaction_id != readback.transaction_id {
        return Err(ConfidentialFundingRefusal::MinedReadbackMismatch {
            fact: MinedReadbackFact::TransactionIdentity,
            index: None,
        });
    }
    if decoded.witness_transaction_id != readback.witness_transaction_id {
        return Err(ConfidentialFundingRefusal::MinedReadbackMismatch {
            fact: MinedReadbackFact::WitnessTransactionIdentity,
            index: None,
        });
    }
    if !oracles.inclusion.contains(
        &readback.block_hash,
        readback.block_height,
        &decoded.transaction_id,
    ) {
        return Err(ConfidentialFundingRefusal::MinedInclusionUnverified);
    }

    let outputs = bind_outputs(exchange, response, &decoded, oracles)?;
    Ok(ConfidentialFundingObservation::Funded(Box::new(
        ReadbackDerivedFunding {
            transaction_id: decoded.transaction_id,
            witness_transaction_id: decoded.witness_transaction_id,
            block_hash: readback.block_hash.clone(),
            block_height: readback.block_height,
            outputs,
        },
    )))
}

/// Every output, in the order the fixture fixed.
fn bind_outputs(
    exchange: &ConfidentialFundingExchange<'_>,
    response: &NativeOperationResponse,
    decoded: &DecodedFundingTransaction,
    oracles: &ConfidentialFundingOracles<'_>,
) -> Result<Vec<ReadbackDerivedOutput>, ConfidentialFundingRefusal> {
    let requested = exchange.requested.destinations.len();
    let observed = response.confidential_funded_outputs.len();
    if requested != observed {
        return Err(ConfidentialFundingRefusal::OutputCountMismatch {
            requested,
            observed,
        });
    }
    let mut seen: BTreeSet<&WireOutpoint> = BTreeSet::new();
    let asset = expected_explicit_asset(exchange.requested, response)?;
    let mut bound = Vec::with_capacity(observed);
    for (index, reported) in response.confidential_funded_outputs.iter().enumerate() {
        if !seen.insert(&reported.outpoint) {
            return Err(ConfidentialFundingRefusal::DuplicateFundedOutpoint {
                outpoint: reported.outpoint.clone(),
            });
        }
        let position = usize::try_from(reported.outpoint.vout)
            .ok()
            .and_then(|vout| decoded.outputs.get(vout))
            .ok_or(ConfidentialFundingRefusal::MinedReadbackMismatch {
                fact: MinedReadbackFact::OutputWitnessIndex,
                index: Some(index),
            })?;
        let binding = OutputBinding {
            index,
            reported,
            decoded: position,
            expected_asset: &asset,
            expected_program: &exchange.requested.destinations[index].output_program,
            transaction_id: &decoded.transaction_id,
        };
        bound.push(binding.bind(oracles.rangeproofs)?);
    }
    Ok(bound)
}
