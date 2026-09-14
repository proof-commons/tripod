//! Static operator key shape and source-selected STATE message coverage.
//!
//! Shape does not establish curve validity, deployment identity, or signature
//! semantics. Deployment or typed STATE policy must commit the key; a request
//! cannot select it. The transaction boundary and native evidence still owe
//! curve and signature checks. The announcement assessment remains external.
//! A profile name is provenance: the reviewed capability must establish each
//! required dimension, and signing must independently realize that message.

use std::collections::{BTreeMap, BTreeSet};

use target_elements::{
    AuthorizationContract, CanonicalEncodingRule, EncodingClass, EncodingDomain, PayloadWidth,
    ReviewedElementsTapscriptDefinition, SighashCapability, SighashDimension,
    TargetContractVersion, UnknownPublicKeyTypeRule,
};

use crate::authorization::{DimensionRefusal, DimensionRole, OutsideMessageGround};
use crate::capability::census_enum;

census_enum! {
    /// Required operator negatives, in the protocol's order.
    pub enum OperatorKeyNegative {
        /// Empty key.
        EmptyKey,
        /// Wrong width.
        WrongWidth,
        /// Malformed approved key.
        MalformedApprovedKey,
        /// Unknown nonempty key type.
        UnknownNonemptyKeyType,
        /// Another operator's approved key.
        ApprovedKeyOfAnotherOperator,
        /// Valid signature under another key.
        ValidSignatureAgainstAnotherKey,
        /// Valid signature over another transaction.
        ValidSignatureOverAnotherTransaction,
    }
}

/// The component that can decide a negative, including retained obligations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperatorKeyFaultClass {
    /// Empty bytes, width, and encoding class are decidable without arithmetic.
    /// Unknown classes cannot be offered through the target's encoding census;
    /// emitted checks must also enforce the closure against raw unknown types.
    KeyShapeGate,
    /// Membership needs curve arithmetic at the transaction boundary and native
    /// evidence. Passing the shape gate cannot discharge this obligation.
    CurveValidityAtTransactionBoundaryAndNativeEvidence,
    /// Identifying another operator requires the committed deployment or typed
    /// STATE policy key, which the candidate deployment binding must supply.
    CommittedDeploymentKey,
    /// Both signature negatives require the exact signing request and native
    /// verification; neither key bytes nor a static profile can decide them.
    SigningBoundaryAndNativeEvidence,
}

impl OperatorKeyNegative {
    /// Classifies every negative by the information needed to decide it.
    #[must_use]
    pub const fn decided_by(self) -> OperatorKeyFaultClass {
        match self {
            Self::EmptyKey | Self::WrongWidth | Self::UnknownNonemptyKeyType => {
                OperatorKeyFaultClass::KeyShapeGate
            }
            Self::MalformedApprovedKey => {
                OperatorKeyFaultClass::CurveValidityAtTransactionBoundaryAndNativeEvidence
            }
            Self::ApprovedKeyOfAnotherOperator => OperatorKeyFaultClass::CommittedDeploymentKey,
            Self::ValidSignatureAgainstAnotherKey | Self::ValidSignatureOverAnotherTransaction => {
                OperatorKeyFaultClass::SigningBoundaryAndNativeEvidence
            }
        }
    }
}

/// What the target's unknown-key rule requires beyond its signature result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperatorKeyObligation {
    /// Authenticate the encoding before trusting a primitive that can succeed
    /// for unknown keys without verifying a signature.
    AuthenticateEncodingIndependently,
    /// The target itself refuses unknown encodings.
    SignatureResultSuffices,
}

/// Operator-specific closure; an owner closure cannot authorize this role.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorKeyEncodingClosure {
    approved: EncodingClass,
    unknown_key_rule: UnknownPublicKeyTypeRule,
    obligation: OperatorKeyObligation,
}

impl OperatorKeyEncodingClosure {
    /// The contract's approved signature public-key encoding.
    #[must_use]
    pub const fn approved(&self) -> EncodingClass {
        self.approved
    }

    /// The primitive's treatment of unknown public-key types.
    #[must_use]
    pub const fn unknown_key_rule(&self) -> UnknownPublicKeyTypeRule {
        self.unknown_key_rule
    }

    /// The encoding check owed before trusting the signature result.
    #[must_use]
    pub const fn obligation(&self) -> OperatorKeyObligation {
        self.obligation
    }

    /// All seven negatives, regardless of the target's unknown-key rule.
    pub fn negatives(&self) -> impl Iterator<Item = OperatorKeyNegative> {
        OperatorKeyNegative::ALL.iter().copied()
    }
}

/// Derives the closure fail-closed: only explicit rejection of unknown keys
/// permits the signature primitive's answer to carry the encoding check.
#[must_use]
pub const fn operator_key_encoding_closure(
    contract: &AuthorizationContract,
) -> OperatorKeyEncodingClosure {
    let signature = contract.signature();
    let unknown_key_rule = signature.unknown_public_key_type();
    OperatorKeyEncodingClosure {
        approved: signature.public_key_encoding(),
        unknown_key_rule,
        obligation: match unknown_key_rule {
            UnknownPublicKeyTypeRule::Rejected => OperatorKeyObligation::SignatureResultSuffices,
            _ => OperatorKeyObligation::AuthenticateEncodingIndependently,
        },
    }
}

/// Shape refusals, in the constructor's decision order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperatorKeyRejection {
    /// No operator bytes were offered.
    OperatorOmitted,
    /// The offered class belongs to a domain other than public keys.
    NotAKeyEncoding {
        /// The offered encoding class.
        offered: EncodingClass,
        /// Its domain, distinguishing a scalar from a public key.
        domain: EncodingDomain,
    },
    /// A public-key encoding other than the approved class was offered.
    AlternateEncodingOfApprovedKey {
        /// The offered encoding class.
        offered: EncodingClass,
        /// The contract's approved class.
        approved: EncodingClass,
    },
    /// The approved class no longer fixes a unique canonical form.
    NonUniqueCanonicalForm {
        /// The approved class.
        approved: EncodingClass,
        /// Its canonicality rule.
        rule: CanonicalEncodingRule,
    },
    /// The approved class no longer fixes an exact payload width.
    UnfixedApprovedWidth {
        /// The approved class.
        approved: EncodingClass,
        /// Its payload width rule.
        width: PayloadWidth,
    },
    /// The offered byte count differs from the approved width.
    WrongWidth {
        /// The offered byte count.
        offered: usize,
        /// The required byte count.
        required: usize,
    },
}

/// Curve membership remains a validation obligation after shape checking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperatorKeyCurveValidity {
    /// This crate performs no curve arithmetic; the transaction boundary and
    /// native evidence must validate the approved encoding's curve point.
    Unverified,
}

/// Public operator key bytes checked only for target-approved encoding shape.
///
/// No conversion to or from an owner key is provided. Deployment must bind
/// these bytes independently of the request before using them for authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorKey {
    encoding: EncodingClass,
    bytes: Vec<u8>,
}

impl OperatorKey {
    /// Checks emptiness, domain, class, canonicality, exact width, then length.
    /// The domain check separates scalar-shaped private material from public
    /// keys; byte width alone cannot make that distinction.
    ///
    /// # Errors
    /// Returns the first applicable [`OperatorKeyRejection`]. Curve membership
    /// and deployment identity remain obligations even when shape succeeds.
    pub fn new(
        closure: &OperatorKeyEncodingClosure,
        encoding: EncodingClass,
        bytes: Vec<u8>,
    ) -> Result<Self, OperatorKeyRejection> {
        if bytes.is_empty() {
            return Err(OperatorKeyRejection::OperatorOmitted);
        }
        let shape = encoding.v1_shape();
        if shape.domain() != EncodingDomain::Key {
            return Err(OperatorKeyRejection::NotAKeyEncoding {
                offered: encoding,
                domain: shape.domain(),
            });
        }
        let approved = closure.approved();
        if encoding != approved {
            return Err(OperatorKeyRejection::AlternateEncodingOfApprovedKey {
                offered: encoding,
                approved,
            });
        }
        let approved_shape = approved.v1_shape();
        if approved_shape.canonicality() != CanonicalEncodingRule::Unique {
            return Err(OperatorKeyRejection::NonUniqueCanonicalForm {
                approved,
                rule: approved_shape.canonicality(),
            });
        }
        let PayloadWidth::Exact(required) = approved_shape.payload() else {
            return Err(OperatorKeyRejection::UnfixedApprovedWidth {
                approved,
                width: approved_shape.payload(),
            });
        };
        if bytes.len() != required.get() {
            return Err(OperatorKeyRejection::WrongWidth {
                offered: bytes.len(),
                required: required.get(),
            });
        }
        Ok(Self { encoding, bytes })
    }

    /// The approved public-key encoding class.
    #[must_use]
    pub const fn encoding(&self) -> EncodingClass {
        self.encoding
    }

    /// The shape-checked public bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The curve-membership obligation that shape checking cannot discharge.
    #[must_use]
    pub const fn curve_validity(&self) -> OperatorKeyCurveValidity {
        OperatorKeyCurveValidity::Unverified
    }
}

census_enum! {
    /// STATE data the operator message must protect, in protocol order.
    ///
    /// Input and spent-output fields have separate carriers; version and lock
    /// time also remain separate so each target dimension is accountable.
    pub enum StateProtectedDatum {
        /// Every input field required by the target.
        InputFields,
        /// Every spent-output field required by the target.
        SpentOutputFields,
        /// Every output.
        EveryOutput,
        /// Successor metadata publication where serialized.
        SuccessorMetadataPublication,
        /// Successor output program.
        SuccessorOutputProgram,
        /// Transaction version.
        TransactionVersion,
        /// Lock time.
        LockTime,
        /// Relevant output witnesses.
        RelevantOutputWitnesses,
        /// Issuance absence.
        IssuanceAbsence,
        /// Executing leaf and script-path terms.
        ExecutingLeafAndScriptPathTerms,
    }
}

/// The reviewed capability's answer, or a refusal to reuse a stale witness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperatorProfileDisposition {
    /// Every required dimension is reviewed; transaction evidence is still owed.
    Established,
    /// The review does not establish these required dimensions.
    ReviewIncomplete {
        /// Missing dimensions, in target census order.
        unreviewed: BTreeSet<SighashDimension>,
    },
    /// The capability about to be consumed differs from the witness's pin.
    StaleRevision {
        /// The revision under which establishment succeeded.
        pinned: TargetContractVersion,
        /// The revision the consumer now offers.
        offered: TargetContractVersion,
    },
}

/// Source-selected coverage, with no public constructor or writable fields.
/// Arbitrary caller-authored sets cannot substitute for the coverage argument.
///
/// ```compile_fail,E0451
/// use std::collections::BTreeMap;
/// use tapscript::OperatorSighashProfile;
/// let forged = OperatorSighashProfile {
///     roles: BTreeMap::new(), coverage: BTreeMap::new(),
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorSighashProfile {
    roles: BTreeMap<SighashDimension, DimensionRole>,
    coverage: BTreeMap<StateProtectedDatum, BTreeSet<SighashDimension>>,
}

impl OperatorSighashProfile {
    /// Every dimension the signature must commit to, in target census order.
    pub fn required(&self) -> impl Iterator<Item = SighashDimension> + '_ {
        self.roles
            .iter()
            .filter(|(_, role)| **role == DimensionRole::Required)
            .map(|(dimension, _)| *dimension)
    }

    /// Every refused dimension with its named ground.
    pub fn refused(&self) -> impl Iterator<Item = (SighashDimension, DimensionRefusal)> + '_ {
        self.roles
            .iter()
            .filter_map(|(dimension, role)| match role {
                DimensionRole::Refused(ground) => Some((*dimension, *ground)),
                DimensionRole::Required | DimensionRole::NotCarriedByTheMessage { .. } => None,
            })
    }

    /// Dimensions protected through a consensus composition outside the message.
    pub fn not_carried_by_the_message(
        &self,
    ) -> impl Iterator<Item = (SighashDimension, SighashDimension, OutsideMessageGround)> + '_ {
        self.roles
            .iter()
            .filter_map(|(dimension, role)| match role {
                DimensionRole::NotCarriedByTheMessage { carried_by, ground } => {
                    Some((*dimension, *carried_by, *ground))
                }
                DimensionRole::Required | DimensionRole::Refused(_) => None,
            })
    }

    /// The selected role of one offered target dimension.
    #[must_use]
    pub fn role(&self, dimension: SighashDimension) -> Option<DimensionRole> {
        self.roles.get(&dimension).copied()
    }

    /// Every message carrier of a STATE datum.
    ///
    /// # Panics
    /// Panics only if source selection omits a census member, which callers
    /// cannot arrange because profiles have no public constructor.
    #[must_use]
    pub fn carrier(&self, datum: StateProtectedDatum) -> &BTreeSet<SighashDimension> {
        self.coverage
            .get(&datum)
            .expect("a selected profile covers every protected datum")
    }

    /// Each STATE datum paired with each dimension carrying it.
    pub fn coverage(&self) -> impl Iterator<Item = (StateProtectedDatum, SighashDimension)> + '_ {
        self.coverage.iter().flat_map(|(datum, dimensions)| {
            dimensions.iter().map(move |dimension| (*datum, *dimension))
        })
    }

    /// Recomputes establishment from the capability; no caller supplies the answer.
    #[must_use]
    pub fn assess(&self, capability: &SighashCapability) -> OperatorProfileDisposition {
        let unreviewed = self
            .required()
            .filter(|dimension| !capability.is_reviewed(*dimension))
            .collect::<BTreeSet<_>>();

        if unreviewed.is_empty() {
            OperatorProfileDisposition::Established
        } else {
            OperatorProfileDisposition::ReviewIncomplete { unreviewed }
        }
    }
}

/// The reviewed all-inputs/all-outputs script-path selection.
///
/// Narrowing forms and issuance are refused for the same reasons as the owner
/// selection. Annex absence remains a signing-boundary obligation: the target
/// offers no annex dimension to classify or review here. The internal key is
/// composed through the spent program and consensus control-block check.
#[must_use]
pub fn selected_operator_profile() -> OperatorSighashProfile {
    use SighashDimension as Dimension;

    let roles = SighashDimension::ALL
        .iter()
        .map(|dimension| {
            let role = match dimension {
                Dimension::SingleOutput => {
                    DimensionRole::Refused(DimensionRefusal::LeavesOtherOutputsFree)
                }
                Dimension::InputExtensionPermitted => {
                    DimensionRole::Refused(DimensionRefusal::LeavesInputSetOpen)
                }
                Dimension::Issuance => {
                    DimensionRole::Refused(DimensionRefusal::SubjectRefusedByTheCensusAndTheDecoder)
                }
                Dimension::InternalKey => DimensionRole::NotCarriedByTheMessage {
                    carried_by: Dimension::SpentOutputs,
                    ground: OutsideMessageGround::ComposedThroughTheControlBlockCheck,
                },
                Dimension::AllOutputs
                | Dimension::AllInputs
                | Dimension::Version
                | Dimension::LockTime
                | Dimension::TapleafHash
                | Dimension::SpentOutputs => DimensionRole::Required,
                _ => DimensionRole::Refused(DimensionRefusal::Unclassified),
            };

            (*dimension, role)
        })
        .collect();

    let coverage = StateProtectedDatum::ALL
        .iter()
        .map(|datum| (*datum, state_carriers(*datum)))
        .collect();
    OperatorSighashProfile { roles, coverage }
}

/// The message argument for each STATE item. Inputs and spent outputs are
/// distinct streams. Metadata and the successor program serialize in the
/// output list; relevant witnesses use the all-outputs output-witness hash.
/// Issuance is refused by the census and decoder, leaving the issuance terms
/// as input-count commitments. The executing leaf uses the tapleaf hash under
/// the fixed, annex-absent script-path terms. Every item has a nonempty carrier;
/// no outside-message datum is silently represented by an empty set.
fn state_carriers(datum: StateProtectedDatum) -> BTreeSet<SighashDimension> {
    use SighashDimension as Dimension;
    BTreeSet::from([match datum {
        StateProtectedDatum::InputFields | StateProtectedDatum::IssuanceAbsence => {
            Dimension::AllInputs
        }
        StateProtectedDatum::SpentOutputFields => Dimension::SpentOutputs,
        StateProtectedDatum::EveryOutput
        | StateProtectedDatum::SuccessorMetadataPublication
        | StateProtectedDatum::SuccessorOutputProgram
        | StateProtectedDatum::RelevantOutputWitnesses => Dimension::AllOutputs,
        StateProtectedDatum::TransactionVersion => Dimension::Version,
        StateProtectedDatum::LockTime => Dimension::LockTime,
        StateProtectedDatum::ExecutingLeafAndScriptPathTerms => Dimension::TapleafHash,
    }])
}

/// A source-selected profile established against one reviewed capability.
///
/// The establishment conclusion is unavailable for callers to state: private
/// fields, no default, and the sole public constructor derive it from review.
/// Every consumer must call [`Self::check_revision`] before relying on it.
///
/// ```compile_fail,E0451
/// use std::collections::BTreeSet;
/// use tapscript::{EstablishedOperatorProfile, selected_operator_profile};
/// use target_elements::TargetContractVersion;
/// let forged = EstablishedOperatorProfile {
///     profile: selected_operator_profile(),
///     established: BTreeSet::new(),
///     capability_revision: TargetContractVersion::V2,
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EstablishedOperatorProfile {
    profile: OperatorSighashProfile,
    established: BTreeSet<SighashDimension>,
    capability_revision: TargetContractVersion,
}

impl EstablishedOperatorProfile {
    /// Derives establishment and its revision from the same reviewed target.
    ///
    /// # Errors
    /// Returns [`OperatorProfileDisposition::ReviewIncomplete`] with every
    /// required dimension the reviewed capability does not establish.
    pub fn establish(
        profile: OperatorSighashProfile,
        target: &ReviewedElementsTapscriptDefinition,
    ) -> Result<Self, OperatorProfileDisposition> {
        Self::establish_against(
            profile,
            target.definition().authorization().sighash(),
            target.definition().version(),
        )
    }

    // Shared assessment core, crate-visible only so degraded-capability tests
    // can exercise refusal without forging the target's reviewed wrapper.
    pub(crate) fn establish_against(
        profile: OperatorSighashProfile,
        capability: &SighashCapability,
        capability_revision: TargetContractVersion,
    ) -> Result<Self, OperatorProfileDisposition> {
        match profile.assess(capability) {
            OperatorProfileDisposition::Established => {
                let established = profile.required().collect();
                Ok(Self {
                    profile,
                    established,
                    capability_revision,
                })
            }
            refusal => Err(refusal),
        }
    }

    /// The selected profile whose required dimensions were established.
    #[must_use]
    pub const fn profile(&self) -> &OperatorSighashProfile {
        &self.profile
    }

    /// The required dimensions established when this witness was minted.
    #[must_use]
    pub const fn established_dimensions(&self) -> &BTreeSet<SighashDimension> {
        &self.established
    }

    /// The capability revision bound to this witness.
    #[must_use]
    pub const fn capability_revision(&self) -> TargetContractVersion {
        self.capability_revision
    }

    /// Checks the revision the consumer is about to rely on against the pin.
    ///
    /// # Errors
    /// Returns [`OperatorProfileDisposition::StaleRevision`] naming both
    /// revisions when the offered capability is not the pinned one.
    pub fn check_revision(
        &self,
        offered: TargetContractVersion,
    ) -> Result<(), OperatorProfileDisposition> {
        if offered == self.capability_revision {
            Ok(())
        } else {
            Err(OperatorProfileDisposition::StaleRevision {
                pinned: self.capability_revision,
                offered,
            })
        }
    }
}

/// Whether every offered target dimension has an explicitly considered role.
#[must_use]
pub fn operator_profile_classifies_every_offered_dimension(
    profile: &OperatorSighashProfile,
) -> bool {
    SighashDimension::ALL.iter().all(|dimension| {
        matches!(profile.role(*dimension), Some(role) if role != DimensionRole::Refused(DimensionRefusal::Unclassified))
    })
}

/// Whether every STATE item has nonempty coverage only on required dimensions.
#[must_use]
pub fn operator_profile_coverage_lands_only_on_required_dimensions(
    profile: &OperatorSighashProfile,
) -> bool {
    StateProtectedDatum::ALL.iter().all(|datum| {
        let carriers = profile.carrier(*datum);
        !carriers.is_empty()
            && carriers
                .iter()
                .all(|dimension| profile.role(*dimension) == Some(DimensionRole::Required))
    })
}
