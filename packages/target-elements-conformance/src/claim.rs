//! The typed evidence-claim census.
//!
//! # Why a broad requirement is not a claim
//!
//! A target evidence requirement names a whole dimension of behaviour —
//! "input introspection semantics", "signature semantics", "issuance
//! introspection". Aggregating cases straight onto such a row is
//! existential: one passing case moves the row to passed, and the report
//! then says the dimension is established when what was established is
//! one corner of it. An issuance-absent case passes the issuance row
//! without any issuing input ever existing; a rejection-only signature
//! suite passes the signature row without one signature ever verifying.
//!
//! A claim is the unit beneath the row: one semantic statement, owned by
//! exactly one requirement, that a case either bears on or does not. A
//! requirement passes only when every claim it *requires* has a passing
//! case bearing on it, and a claim no passing case bears on stays
//! unresolved and says so.
//!
//! # The registry is first-party policy
//!
//! Which claims exist, which requirement owns each, and which Guide 10
//! requires are decisions this repository makes and states here. No
//! executor supplies them and none can widen or narrow them. What an
//! executor influences is only whether the cases bearing on a claim
//! passed.
//!
//! # Claims are derived from the fixture, not asserted beside it
//!
//! [`claims_of`] reads the fixture: its group, its primitive, the layer
//! its verdict is stated at, the leaf-version status, the outcome the
//! contract requires, and the transaction context's own field forms. A
//! hand-written label per case would be a second statement of what the
//! case is, free to drift from what the case actually does — and the
//! drift would always be in the direction of claiming more.
//!
//! This is also what keeps the uncovered claims honest. There is no
//! issuing input in the census, so no case derives an issuance-present
//! claim; there is no accepting transaction-signature fixture, so none
//! derives the accepting signature claim; there is no confidential field,
//! so none derives a confidential-form claim. Those claims exist, are
//! unresolved, and state why.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use target_elements::{OpcodeId, TargetEvidenceRequirementId};

use crate::fixture::{
    EnforcementLayer, ExpectedPrimitiveOutcome, LeafVersionStatus, NativeCaseGroup,
    PrimitiveExecutionContext, PrimitiveFixture,
};
use crate::protocol::ObservedFailureClass;

/// One semantic statement beneath a broad evidence requirement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum NativeEvidenceClaim {
    /// The reviewed semantics apply under the reviewed leaf version.
    ReviewedDomainExecutes,
    /// A byte the contract has not reviewed does not select them.
    UnreviewedLeafSuspendsSemantics,
    /// The reviewed leaf version selects the reviewed semantics.
    ReviewedLeafVersionSelectsDomain,

    /// A reviewed primitive completed and produced its stated result.
    PrimitiveSuccessObserved,
    /// A reviewed primitive aborted in a stated failure class.
    PrimitiveAbortObserved,
    /// A reviewed byte decoded as the reviewed primitive.
    InstructionDecodesAsPrimitive,
    /// A byte outside the reviewed census did not decode as one.
    UnreviewedInstructionRefused,

    /// A literal push in a reviewed form was executed.
    PushFormAccepted,
    /// A malformed literal push was refused.
    PushMalformedRefused,
    /// Relay policy refused a nonminimal push form.
    PushNonminimalRelayRefused,

    /// An input field was read in its explicit form.
    InputFieldExplicitForm,
    /// An input field was read in a confidential form.
    InputFieldConfidentialForm,
    /// An output field was read in its explicit form.
    OutputFieldExplicitForm,
    /// An output field was read in a confidential form.
    OutputFieldConfidentialForm,
    /// A whole-transaction field was read.
    TransactionFieldObserved,

    /// An input carrying no issuance reported the absent form.
    IssuanceAbsentObserved,
    /// An input carrying an issuance reported the present form.
    IssuancePresentObserved,
    /// An input carrying a reissuance reported it as one.
    ReissuancePresentObserved,

    /// A signed fixed-width operation completed.
    ArithmeticOperationCompleted,
    /// A signed fixed-width operation was refused before completing.
    ArithmeticOperandRefused,
    /// A comparison completed and answered true.
    ComparisonAnsweredTrue,
    /// A comparison completed and answered false.
    ComparisonAnsweredFalse,
    /// A conversion completed.
    ConversionCompleted,
    /// A conversion was refused.
    ConversionRefused,

    /// A streaming hash reproduced a published vector.
    StreamingHashVectorReproduced,
    /// A streaming hash refused a malformed context or write.
    StreamingHashContextRefused,

    /// An elliptic-curve relation held.
    CurveRelationHeld,
    /// An elliptic-curve relation was refused.
    CurveRelationRefused,

    /// A signature over a stack message verified.
    StackMessageSignatureAccepted,
    /// A signature over a stack message was refused.
    StackMessageSignatureRefused,
    /// A signature over the transaction sighash verified.
    TransactionSignatureAccepted,
    /// A signature over the transaction sighash was refused.
    TransactionSignatureRefused,
    /// A nonempty public key of an unrecognized form succeeded without
    /// any signature being verified.
    UnknownPublicKeyTypeSucceededUnverified,

    /// A relative timelock was satisfied.
    RelativeTimelockSatisfied,
    /// A relative timelock was not satisfied.
    RelativeTimelockUnsatisfied,

    /// The target's own resource boundary was observed.
    ConsensusResourceBoundObserved,
    /// A node's relay resource boundary was observed.
    PolicyResourceBoundObserved,

    /// Confidential values were observed conserved across a
    /// transaction.
    ConfidentialValueConservationObserved,

    /// The sighash's committed content was observed.
    SighashCommitmentObserved,
    /// A commitment equality was observed.
    CommitmentEqualityObserved,
}

/// Whether Guide 10 requires a claim, and why it does not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ClaimRequirement {
    /// A passing case must bear on this claim.
    Required,
    /// No case in this census can bear on it, for this stated reason.
    ///
    /// Unresolved is not success and is not failure. It is the project
    /// saying which corner of a dimension it has not established, in a
    /// form a reader can enumerate rather than infer from an absence.
    Unresolved(&'static str),
}

/// One claim's owner, requirement, and reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimRecord {
    claim: NativeEvidenceClaim,
    requirement: TargetEvidenceRequirementId,
    status: ClaimRequirement,
}

impl ClaimRecord {
    /// The claim.
    #[must_use]
    pub const fn claim(&self) -> NativeEvidenceClaim {
        self.claim
    }

    /// The evidence requirement that owns it.
    #[must_use]
    pub const fn requirement(&self) -> TargetEvidenceRequirementId {
        self.requirement
    }

    /// Whether Guide 10 requires it, and why it does not.
    #[must_use]
    pub const fn status(&self) -> ClaimRequirement {
        self.status
    }

    /// Whether a passing case must bear on it.
    #[must_use]
    pub const fn is_required(&self) -> bool {
        matches!(self.status, ClaimRequirement::Required)
    }

    /// Why the claim is unresolved, where it is.
    #[must_use]
    pub const fn unresolved_reason(&self) -> Option<&'static str> {
        match self.status {
            ClaimRequirement::Required => None,
            ClaimRequirement::Unresolved(reason) => Some(reason),
        }
    }
}

/// The complete claim census.
///
/// Every claim appears exactly once, owned by exactly one requirement.
/// There is no wildcard arm: a claim admitted later must be written here,
/// which is how a new semantic statement fails to disappear into an
/// existing family.
const CLAIM_CENSUS: &[ClaimRecord] = {
    use ClaimRequirement::{Required, Unresolved};
    use NativeEvidenceClaim as Claim;
    use TargetEvidenceRequirementId as Requirement;

    const fn record(
        claim: NativeEvidenceClaim,
        requirement: TargetEvidenceRequirementId,
        status: ClaimRequirement,
    ) -> ClaimRecord {
        ClaimRecord {
            claim,
            requirement,
            status,
        }
    }

    &[
        record(
            Claim::ReviewedDomainExecutes,
            Requirement::TapscriptExecutionDomain,
            Required,
        ),
        record(
            Claim::UnreviewedLeafSuspendsSemantics,
            Requirement::LeafVersionActivation,
            Required,
        ),
        record(
            Claim::ReviewedLeafVersionSelectsDomain,
            Requirement::LeafVersionActivation,
            Required,
        ),
        record(
            Claim::PrimitiveSuccessObserved,
            Requirement::OpcodeSemantics,
            Required,
        ),
        record(
            Claim::PrimitiveAbortObserved,
            Requirement::OpcodeSemantics,
            Required,
        ),
        record(
            Claim::InstructionDecodesAsPrimitive,
            Requirement::OpcodeSemantics,
            Required,
        ),
        record(
            Claim::UnreviewedInstructionRefused,
            Requirement::OpcodeSemantics,
            Required,
        ),
        record(
            Claim::PushFormAccepted,
            Requirement::PushEncodingSemantics,
            Required,
        ),
        record(
            Claim::PushMalformedRefused,
            Requirement::PushEncodingSemantics,
            Required,
        ),
        record(
            Claim::PushNonminimalRelayRefused,
            Requirement::PushEncodingSemantics,
            Unresolved(
                "the reviewed push contract classes nonminimal form as a relay rule, and the \
                 census states its nonminimal forms at the consensus layer, where the target \
                 accepts them",
            ),
        ),
        record(
            Claim::InputFieldExplicitForm,
            Requirement::InputIntrospectionSemantics,
            Required,
        ),
        record(
            Claim::InputFieldConfidentialForm,
            Requirement::InputIntrospectionSemantics,
            Unresolved(
                "the reviewed executor cannot yet materialize a blinded asset, value, or nonce, \
                 so no case reads an input field in a confidential form",
            ),
        ),
        record(
            Claim::OutputFieldExplicitForm,
            Requirement::OutputIntrospectionSemantics,
            Required,
        ),
        record(
            Claim::OutputFieldConfidentialForm,
            Requirement::OutputIntrospectionSemantics,
            Unresolved(
                "the reviewed executor cannot yet materialize a blinded asset, value, or nonce, \
                 so no case reads an output field in a confidential form",
            ),
        ),
        record(
            Claim::TransactionFieldObserved,
            Requirement::TransactionIntrospectionSemantics,
            Required,
        ),
        record(
            Claim::IssuanceAbsentObserved,
            Requirement::IssuanceIntrospection,
            Required,
        ),
        record(
            Claim::IssuancePresentObserved,
            Requirement::IssuanceIntrospection,
            Unresolved(
                "the census materializes no issuing input, so the issuance-present stack form, \
                 the issued amounts, and the entropy are not established",
            ),
        ),
        record(
            Claim::ReissuancePresentObserved,
            Requirement::IssuanceIntrospection,
            Unresolved(
                "the census materializes no reissuing input, so the nonzero blinding nonce that \
                 distinguishes reissuance from issuance is not established",
            ),
        ),
        record(
            Claim::ArithmeticOperationCompleted,
            Requirement::ArithmeticSemantics,
            Required,
        ),
        record(
            Claim::ArithmeticOperandRefused,
            Requirement::ArithmeticSemantics,
            Required,
        ),
        record(
            Claim::ComparisonAnsweredTrue,
            Requirement::ComparisonSemantics,
            Required,
        ),
        record(
            Claim::ComparisonAnsweredFalse,
            Requirement::ComparisonSemantics,
            Required,
        ),
        record(
            Claim::ConversionCompleted,
            Requirement::ConversionSemantics,
            Required,
        ),
        record(
            Claim::ConversionRefused,
            Requirement::ConversionSemantics,
            Required,
        ),
        record(
            Claim::StreamingHashVectorReproduced,
            Requirement::StreamingHashSemantics,
            Required,
        ),
        record(
            Claim::StreamingHashContextRefused,
            Requirement::StreamingHashSemantics,
            Required,
        ),
        record(
            Claim::CurveRelationHeld,
            Requirement::EllipticCurveSemantics,
            Required,
        ),
        record(
            Claim::CurveRelationRefused,
            Requirement::EllipticCurveSemantics,
            Required,
        ),
        record(
            Claim::StackMessageSignatureAccepted,
            Requirement::SignatureSemantics,
            Required,
        ),
        record(
            Claim::StackMessageSignatureRefused,
            Requirement::SignatureSemantics,
            Required,
        ),
        record(
            Claim::TransactionSignatureRefused,
            Requirement::SignatureSemantics,
            Required,
        ),
        record(
            Claim::TransactionSignatureAccepted,
            Requirement::SignatureSemantics,
            Unresolved(
                "a signature that verifies against a transaction sighash commits to the \
                 transaction the executor builds, which no static fixture can state, so only \
                 the rejecting paths of the transaction-signature primitives are established",
            ),
        ),
        record(
            Claim::UnknownPublicKeyTypeSucceededUnverified,
            Requirement::SignatureSemantics,
            Required,
        ),
        record(
            Claim::RelativeTimelockSatisfied,
            Requirement::RelativeTimelockSemantics,
            Required,
        ),
        record(
            Claim::RelativeTimelockUnsatisfied,
            Requirement::RelativeTimelockSemantics,
            Required,
        ),
        record(
            Claim::ConsensusResourceBoundObserved,
            Requirement::ConsensusResourceLimits,
            Required,
        ),
        record(
            Claim::PolicyResourceBoundObserved,
            Requirement::PolicyResourceLimits,
            Unresolved(
                "every resource case is stated at the consensus layer, and a consensus \
                 acceptance is not evidence that relay policy refuses the same transaction",
            ),
        ),
        record(
            Claim::ConfidentialValueConservationObserved,
            Requirement::ConfidentialValueConservation,
            Unresolved(
                "value conservation is a whole-transaction property, which needs complete \
                 transaction evidence rather than a script-level case",
            ),
        ),
        record(
            Claim::SighashCommitmentObserved,
            Requirement::SighashSemantics,
            Unresolved(
                "no sighash dimension is promoted to reviewed in the static contract, so there \
                 is no reviewed statement for a case to be stated against",
            ),
        ),
        record(
            Claim::CommitmentEqualityObserved,
            Requirement::CommitmentEquality,
            Unresolved(
                "the reviewed contract describes no target mechanism for commitment equality, \
                 and the low-level curve and hash primitives are not one",
            ),
        ),
    ]
};

/// The typed claim census, indexed by claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimRegistry {
    records: BTreeMap<NativeEvidenceClaim, ClaimRecord>,
}

impl ClaimRegistry {
    /// One claim's record.
    #[must_use]
    pub fn record(&self, claim: NativeEvidenceClaim) -> Option<&ClaimRecord> {
        self.records.get(&claim)
    }

    /// Every claim, in claim order.
    pub fn iter(&self) -> impl Iterator<Item = &ClaimRecord> {
        self.records.values()
    }

    /// The claims one requirement owns, in claim order.
    #[must_use]
    pub fn owned_by(
        &self,
        requirement: TargetEvidenceRequirementId,
    ) -> BTreeSet<NativeEvidenceClaim> {
        self.records
            .values()
            .filter(|record| record.requirement == requirement)
            .map(|record| record.claim)
            .collect()
    }

    /// The claims one requirement requires a passing case for.
    #[must_use]
    pub fn required_claims(
        &self,
        requirement: TargetEvidenceRequirementId,
    ) -> BTreeSet<NativeEvidenceClaim> {
        self.records
            .values()
            .filter(|record| record.requirement == requirement && record.is_required())
            .map(|record| record.claim)
            .collect()
    }

    /// How many claims the census holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the census is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// The first-party claim census.
///
/// # Errors
///
/// [`NativeConformanceError::DuplicateEvidenceClaim`] when one claim is
/// stated twice, and
/// [`NativeConformanceError::EvidenceCensusMismatch`] when a claim names
/// a requirement outside the target's evidence census.
pub fn claim_registry() -> Result<ClaimRegistry, crate::error::NativeConformanceError> {
    let mut records = BTreeMap::new();
    for record in CLAIM_CENSUS {
        if !TargetEvidenceRequirementId::ALL.contains(&record.requirement) {
            return Err(crate::error::NativeConformanceError::EvidenceCensusMismatch);
        }
        if records.insert(record.claim, *record).is_some() {
            return Err(crate::error::NativeConformanceError::DuplicateEvidenceClaim(record.claim));
        }
    }
    Ok(ClaimRegistry { records })
}

/// How far the case's primitives got.
///
/// The middle arm is the one that matters: a comparison answering false
/// and an operation that retained its operands are *successful*
/// primitives whose result the domain's final-stack rule then refuses.
/// Folding them into the abort arm would describe a working primitive as
/// a broken one.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CaseOutcome {
    /// The spend is valid.
    Accepted,
    /// Every primitive completed, and the spend is invalid anyway.
    CompletedAndRejected,
    /// A primitive aborted.
    Aborted,
}

/// What one fixture states, gathered once for the claim predicates.
struct CaseFacts<'a> {
    group: NativeCaseGroup,
    opcode: Option<OpcodeId>,
    relay: bool,
    reviewed_leaf: bool,
    outcome: CaseOutcome,
    aborted: BTreeSet<ObservedFailureClass>,
    context: Option<&'a PrimitiveExecutionContext>,
    initial_stack: &'a [Vec<u8>],
}

impl CaseFacts<'_> {
    /// Whether the spend is valid.
    const fn accepting(&self) -> bool {
        matches!(self.outcome, CaseOutcome::Accepted)
    }

    /// Whether every primitive ran to completion.
    const fn completed(&self) -> bool {
        !matches!(self.outcome, CaseOutcome::Aborted)
    }
}

/// Which claims one fixture bears on.
///
/// Read from the fixture itself, so a case cannot claim more than it
/// executes. Every predicate below is a statement about the exact script,
/// stack, context, layer, and stated outcome the executor is handed.
#[must_use]
pub fn claims_of(fixture: &PrimitiveFixture) -> BTreeSet<NativeEvidenceClaim> {
    let expected = fixture.expected();
    let facts = CaseFacts {
        group: fixture.case().group(),
        opcode: fixture.case().opcode(),
        relay: fixture.enforcement_layer() == EnforcementLayer::RelayPolicy,
        reviewed_leaf: fixture.leaf_version_status() == LeafVersionStatus::Reviewed,
        outcome: outcome_of(expected),
        aborted: abort_classes(expected),
        context: fixture.context(),
        initial_stack: fixture.initial_stack(),
    };

    let mut claims = BTreeSet::new();
    claims.extend(activation_claims(&facts));
    claims.extend(primitive_claims(&facts));
    claims.extend(introspection_claims(&facts));
    claims.extend(numeric_claims(&facts));
    claims.extend(authorization_claims(&facts));
    claims.extend(boundary_claims(&facts));
    claims
}

/// The domain and leaf-activation claims.
fn activation_claims(facts: &CaseFacts<'_>) -> BTreeSet<NativeEvidenceClaim> {
    use NativeEvidenceClaim as Claim;

    let mut claims = BTreeSet::new();
    if facts.group == NativeCaseGroup::ExecutionDomain && facts.reviewed_leaf {
        claims.insert(Claim::ReviewedDomainExecutes);
    }
    if facts.group == NativeCaseGroup::LeafVersion {
        claims.insert(if facts.reviewed_leaf {
            Claim::ReviewedLeafVersionSelectsDomain
        } else {
            Claim::UnreviewedLeafSuspendsSemantics
        });
    }
    claims
}

/// The primitive-semantics and literal-push claims.
///
/// A case naming a reviewed primitive says that the byte executed at all,
/// which is the opcode row's subject.
fn primitive_claims(facts: &CaseFacts<'_>) -> BTreeSet<NativeEvidenceClaim> {
    use NativeEvidenceClaim as Claim;

    let mut claims = BTreeSet::new();
    if facts.opcode.is_some() && facts.reviewed_leaf {
        if facts.completed() {
            claims.insert(Claim::PrimitiveSuccessObserved);
        }
        if !facts.aborted.is_empty() {
            claims.insert(Claim::PrimitiveAbortObserved);
        }
    }
    if facts.group == NativeCaseGroup::InstructionEncoding {
        if facts.aborted.contains(&ObservedFailureClass::UnknownOpcode) {
            claims.insert(Claim::UnreviewedInstructionRefused);
        } else if facts.opcode.is_some() {
            claims.insert(Claim::InstructionDecodesAsPrimitive);
        }
    }
    if facts.group == NativeCaseGroup::PushEncoding {
        claims.insert(if facts.relay {
            Claim::PushNonminimalRelayRefused
        } else if facts.aborted.contains(&ObservedFailureClass::MalformedPush) {
            Claim::PushMalformedRefused
        } else {
            Claim::PushFormAccepted
        });
    }
    claims
}

/// The introspection and issuance claims.
///
/// The confidential forms are derived from the context's own field
/// prefixes, so a census carrying only explicit fields cannot produce
/// one; the issuance forms are derived from what the inputs carry, so a
/// census with no issuing input cannot produce those either.
fn introspection_claims(facts: &CaseFacts<'_>) -> BTreeSet<NativeEvidenceClaim> {
    use NativeEvidenceClaim as Claim;

    let mut claims = BTreeSet::new();
    let confidential = facts.context.is_some_and(carries_confidential_field);
    if facts.group == NativeCaseGroup::InputIntrospection {
        claims.insert(if confidential {
            Claim::InputFieldConfidentialForm
        } else {
            Claim::InputFieldExplicitForm
        });
    }
    if facts.group == NativeCaseGroup::OutputIntrospection {
        claims.insert(if confidential {
            Claim::OutputFieldConfidentialForm
        } else {
            Claim::OutputFieldExplicitForm
        });
    }
    if facts.group == NativeCaseGroup::TransactionIntrospection {
        claims.insert(Claim::TransactionFieldObserved);
    }
    if facts.group == NativeCaseGroup::Issuance {
        claims.insert(match issuance_form(facts.context) {
            IssuanceForm::Absent => Claim::IssuanceAbsentObserved,
            IssuanceForm::Issuance => Claim::IssuancePresentObserved,
            IssuanceForm::Reissuance => Claim::ReissuancePresentObserved,
        });
    }
    claims
}

/// The arithmetic, comparison, conversion, and hashing claims.
fn numeric_claims(facts: &CaseFacts<'_>) -> BTreeSet<NativeEvidenceClaim> {
    use NativeEvidenceClaim as Claim;

    let mut claims = BTreeSet::new();
    let mut pair = |group, success, refusal| {
        if facts.group == group {
            if facts.completed() {
                claims.insert(success);
            }
            if !facts.aborted.is_empty() {
                claims.insert(refusal);
            }
        }
    };
    pair(
        NativeCaseGroup::Arithmetic,
        Claim::ArithmeticOperationCompleted,
        Claim::ArithmeticOperandRefused,
    );
    pair(
        NativeCaseGroup::Conversion,
        Claim::ConversionCompleted,
        Claim::ConversionRefused,
    );
    pair(
        NativeCaseGroup::StreamingHash,
        Claim::StreamingHashVectorReproduced,
        Claim::StreamingHashContextRefused,
    );

    // A comparison that answers false is a *successful* comparison, so
    // the two claims split on the answer rather than on the verdict.
    if facts.group == NativeCaseGroup::Comparison {
        if facts.accepting() {
            claims.insert(Claim::ComparisonAnsweredTrue);
        } else if facts.aborted.is_empty() {
            claims.insert(Claim::ComparisonAnsweredFalse);
        }
    }
    claims
}

/// The curve, signature, and timelock claims.
fn authorization_claims(facts: &CaseFacts<'_>) -> BTreeSet<NativeEvidenceClaim> {
    use NativeEvidenceClaim as Claim;

    let mut claims = BTreeSet::new();
    if facts.group == NativeCaseGroup::EllipticCurve {
        if facts.completed() {
            claims.insert(Claim::CurveRelationHeld);
        }
        if !facts.aborted.is_empty() {
            claims.insert(Claim::CurveRelationRefused);
        }
    }
    // Signatures split by where the signed message comes from: a stack
    // message is a fixture's own value, and a transaction sighash commits
    // to the transaction the executor builds.
    if facts.group == NativeCaseGroup::Signature {
        let stack_message = matches!(
            facts.opcode,
            Some(OpcodeId::CheckSigFromStack | OpcodeId::CheckSigFromStackVerify)
        );
        // The forward-compatibility path is not a signature that
        // verified, and must not be counted as one. Upstream settles the
        // key's width before any verification happens: a nonempty key
        // that is not the recognized width reaches no check at all, so
        // an accepting case here establishes that the target succeeded
        // *without* verifying — the opposite of what the
        // signature-accepted claims say.
        //
        // Otherwise the split is on the *spend*, not on whether the
        // primitive completed: an empty signature makes the checking
        // form consume its operands and push a false, which is a
        // completed primitive and emphatically not a signature that
        // verified.
        if offers_unknown_key_type(facts) && facts.accepting() {
            claims.insert(Claim::UnknownPublicKeyTypeSucceededUnverified);
        } else {
            claims.insert(match (stack_message, facts.accepting()) {
                (true, true) => Claim::StackMessageSignatureAccepted,
                (true, false) => Claim::StackMessageSignatureRefused,
                (false, true) => Claim::TransactionSignatureAccepted,
                (false, false) => Claim::TransactionSignatureRefused,
            });
        }
    }
    if facts.group == NativeCaseGroup::RelativeTimelock {
        claims.insert(if facts.completed() {
            Claim::RelativeTimelockSatisfied
        } else {
            Claim::RelativeTimelockUnsatisfied
        });
    }
    claims
}

/// The resource-boundary and deferred whole-transaction claims.
fn boundary_claims(facts: &CaseFacts<'_>) -> BTreeSet<NativeEvidenceClaim> {
    use NativeEvidenceClaim as Claim;

    let mut claims = BTreeSet::new();
    // Split by the layer the verdict is stated at: a consensus
    // acceptance is not evidence that relay policy refuses the same
    // transaction.
    if facts.group == NativeCaseGroup::Resource {
        claims.insert(if facts.relay {
            Claim::PolicyResourceBoundObserved
        } else {
            Claim::ConsensusResourceBoundObserved
        });
    }
    if facts.group == NativeCaseGroup::ConfidentialValue {
        claims.insert(Claim::ConfidentialValueConservationObserved);
    }
    if facts.group == NativeCaseGroup::Sighash {
        claims.insert(Claim::SighashCommitmentObserved);
    }
    claims
}

/// The width of the one public-key encoding the target verifies
/// against.
///
/// Stated here rather than read from the reviewed contract so that this
/// derivation stays an independent statement about the fixtures. A weld
/// test requires it to agree with the contract, which is what keeps the
/// independence from becoming a second source of truth.
const RECOGNIZED_PUBLIC_KEY_BYTES: usize = 32;

/// Whether the case offers a public key of a form the target does not
/// recognize.
///
/// The key is the topmost operand of every reviewed signature
/// primitive, so the top of the stated initial stack is the key the
/// executor will hand it. An empty key is not this: the target refuses
/// emptiness outright, and folding the two together would put the
/// hardest rejection and the forward-compatibility success under one
/// name.
fn offers_unknown_key_type(facts: &CaseFacts<'_>) -> bool {
    facts
        .initial_stack
        .last()
        .is_some_and(|key| !key.is_empty() && key.len() != RECOGNIZED_PUBLIC_KEY_BYTES)
}

/// The failure classes that are aborts of a primitive.
///
/// A completed evaluation whose one item is false, and a completed
/// evaluation that left the wrong number of items, are *successful*
/// primitives that the domain's final-stack rule then refuses. Counting
/// either as an abort would describe the contract wrongly in the
/// direction that matters: it would make a working primitive look broken.
fn abort_classes(expected: &ExpectedPrimitiveOutcome) -> BTreeSet<ObservedFailureClass> {
    expected
        .classes()
        .into_iter()
        .filter(|class| {
            !matches!(
                class,
                ObservedFailureClass::EvaluatedFalse | ObservedFailureClass::NonSingletonFinalStack
            )
        })
        .collect()
}

/// How far the case's primitives got.
fn outcome_of(expected: &ExpectedPrimitiveOutcome) -> CaseOutcome {
    if expected.is_accepting() {
        CaseOutcome::Accepted
    } else if abort_classes(expected).is_empty() {
        CaseOutcome::CompletedAndRejected
    } else {
        CaseOutcome::Aborted
    }
}

/// Which issuance form a context's inputs carry.
enum IssuanceForm {
    /// No input carries an issuance.
    Absent,
    /// An input carries an issuance, marked by a zero blinding nonce.
    Issuance,
    /// An input carries a reissuance, marked by a nonzero one.
    Reissuance,
}

/// The issuance form one context carries.
fn issuance_form(context: Option<&PrimitiveExecutionContext>) -> IssuanceForm {
    let Some(context) = context else {
        return IssuanceForm::Absent;
    };
    let mut form = IssuanceForm::Absent;
    for input in &context.inputs {
        if let Some(issuance) = input.issuance.as_ref() {
            // A zero blinding nonce marks an issuance and a nonzero one a
            // reissuance, which is the target's own rule.
            form = if issuance.blinding_nonce == [0_u8; 32] {
                IssuanceForm::Issuance
            } else {
                IssuanceForm::Reissuance
            };
        }
    }
    form
}

/// The prefix byte an explicit asset, value, or nonce field carries.
const EXPLICIT_FIELD_PREFIX: u8 = 0x01;

/// The prefix byte an absent field carries.
///
/// An absent nonce is spelled with this prefix and no payload. It is not
/// a confidential form: nothing is hidden by a field that is not there,
/// and reading it as one would make every explicit-only case look like
/// evidence about blinded fields.
const ABSENT_FIELD_PREFIX: u8 = 0x00;

/// Whether any stated field of the context is in a confidential form.
///
/// A field the fixture leaves to the executor states nothing about its
/// form and is not read here, and neither is the absent form.
fn carries_confidential_field(context: &PrimitiveExecutionContext) -> bool {
    let mut fields: Vec<&[u8]> = Vec::new();
    for input in &context.inputs {
        for field in [input.spent_asset.as_deref(), input.spent_value.as_deref()]
            .into_iter()
            .flatten()
        {
            fields.push(field);
        }
        if let Some(issuance) = input.issuance.as_ref() {
            fields.push(&issuance.asset_amount);
            fields.push(&issuance.inflation_keys_amount);
        }
    }
    for output in &context.outputs {
        if let Some(asset) = output.asset.as_deref() {
            fields.push(asset);
        }
        fields.push(&output.value);
        if !output.nonce.is_empty() {
            fields.push(&output.nonce);
        }
    }
    fields.into_iter().any(|field| {
        field.first().is_some_and(|prefix| {
            *prefix != EXPLICIT_FIELD_PREFIX && *prefix != ABSENT_FIELD_PREFIX
        })
    })
}
