//! The typed target definition, its validator, and its stable
//! projection.
//!
//! # Validation precedes consumption
//!
//! A [`TargetDefinition`] has private fields and no public literal
//! form. The only way a consumer obtains one is wrapped in a
//! [`ValidatedTargetDefinition`], which is produced either by the
//! built-in reviewed declaration or by running the validator over an
//! offered definition. A downstream adapter or deployment binding
//! accepts only the wrapper, so there is no path by which a
//! half-specified contract reaches a consumer that would then have to
//! decide what to do about it.
//!
//! # The validator reports everything
//!
//! [`validate_target_definition`] returns *all* diagnostics rather
//! than the first. A contract with three transcription defects should
//! surface three defects, because fixing them one build at a time
//! wastes the reviewer's attention and hides how badly the
//! transcription went.

use std::collections::{BTreeMap, BTreeSet};

use crate::authorization::{AuthorizationContract, reviewed_authorization};
use crate::capability::{
    CapabilityContract, ElementsCapability, prerequisite_cycle_residual, reviewed_capabilities,
};
use crate::confidential::{
    ConfidentialValueContract, IssuanceContract, reviewed_confidential_values, reviewed_issuance,
};
use crate::encoding::{EncodingClass, EncodingSpec, reviewed_encodings};
use crate::error::TargetError;
use crate::evidence::TargetEvidenceRequirementId;
use crate::evidence_registry::{TargetEvidenceRequirement, reviewed_evidence_requirements};
use crate::opcode::{ExecutionDomain, LeafVersion, OpcodeId, OpcodeSpec, reviewed_opcodes};
use crate::resource::{ResourceContract, reviewed_resources};
use crate::success::SuccessContractDefect;

/// The revision of the typed target compatibility contract.
///
/// This is a stable typed key, not a digest. It changes when the
/// semantic shape or interpretation of the contract changes, and a
/// consumer accepts or rejects it as an explicit compatibility
/// decision. Editorial review provenance — which upstream revision was
/// consulted, when, by whom — never moves it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TargetContractVersion(u32);

impl TargetContractVersion {
    /// The first typed contract revision.
    pub const V1: Self = Self(1);

    /// Every contract revision this crate implements.
    pub const SUPPORTED: &'static [Self] = &[Self::V1];

    /// Accepts a contract version number this crate implements.
    ///
    /// # Errors
    ///
    /// Returns [`TargetError::UnsupportedTargetContractVersion`] when
    /// the offered number names no revision in [`Self::SUPPORTED`].
    pub fn supported(value: u32) -> Result<Self, TargetError> {
        let candidate = Self(value);
        if Self::SUPPORTED.contains(&candidate) {
            Ok(candidate)
        } else {
            Err(TargetError::UnsupportedTargetContractVersion { offered: value })
        }
    }

    /// The contract revision number.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// The typed compatibility contract offered for validation.
///
/// Fields are private and there is no public literal form. Build one
/// with [`TargetDefinition::new`] and hand it to
/// [`validate_target_definition`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetDefinition {
    version: TargetContractVersion,
    execution_domain: ExecutionDomain,
    leaf_version: LeafVersion,
    opcodes: BTreeMap<OpcodeId, OpcodeSpec>,
    encodings: BTreeMap<EncodingClass, EncodingSpec>,
    authorization: AuthorizationContract,
    confidential_values: ConfidentialValueContract,
    issuance: IssuanceContract,
    resources: ResourceContract,
    capabilities: BTreeMap<ElementsCapability, CapabilityContract>,
    evidence_requirements: BTreeMap<TargetEvidenceRequirementId, TargetEvidenceRequirement>,
}

/// The parts of a target definition, gathered for construction.
///
/// A struct rather than a long parameter list: eleven positional
/// arguments of which several are maps would make a transposition
/// silent, and the whole point of this type model is that a
/// transposition should be loud.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetDefinitionParts {
    /// The contract revision.
    pub version: TargetContractVersion,
    /// The domain the contract describes.
    pub execution_domain: ExecutionDomain,
    /// The leaf version the contract requires.
    pub leaf_version: LeafVersion,
    /// The reviewed primitive registry.
    pub opcodes: BTreeMap<OpcodeId, OpcodeSpec>,
    /// The reviewed encoding registry.
    pub encodings: BTreeMap<EncodingClass, EncodingSpec>,
    /// Signature, sighash, and timelock dimensions.
    pub authorization: AuthorizationContract,
    /// Confidential-value capabilities.
    pub confidential_values: ConfidentialValueContract,
    /// Issuance and reissuance facts.
    pub issuance: IssuanceContract,
    /// Consensus and policy resource interfaces.
    pub resources: ResourceContract,
    /// The capability registry.
    pub capabilities: BTreeMap<ElementsCapability, CapabilityContract>,
    /// The evidence-requirement registry.
    pub evidence_requirements: BTreeMap<TargetEvidenceRequirementId, TargetEvidenceRequirement>,
}

impl TargetDefinition {
    /// Assembles an unvalidated contract.
    ///
    /// The value this returns carries no guarantee whatever. It is an
    /// input to the validator, and nothing downstream accepts it.
    #[must_use]
    pub fn new(parts: TargetDefinitionParts) -> Self {
        Self {
            version: parts.version,
            execution_domain: parts.execution_domain,
            leaf_version: parts.leaf_version,
            opcodes: parts.opcodes,
            encodings: parts.encodings,
            authorization: parts.authorization,
            confidential_values: parts.confidential_values,
            issuance: parts.issuance,
            resources: parts.resources,
            capabilities: parts.capabilities,
            evidence_requirements: parts.evidence_requirements,
        }
    }

    /// The contract revision.
    #[must_use]
    pub const fn version(&self) -> TargetContractVersion {
        self.version
    }

    /// The domain the contract describes.
    #[must_use]
    pub const fn execution_domain(&self) -> ExecutionDomain {
        self.execution_domain
    }

    /// The leaf version the contract requires.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// The reviewed primitive registry.
    #[must_use]
    pub const fn opcodes(&self) -> &BTreeMap<OpcodeId, OpcodeSpec> {
        &self.opcodes
    }

    /// The reviewed encoding registry.
    #[must_use]
    pub const fn encodings(&self) -> &BTreeMap<EncodingClass, EncodingSpec> {
        &self.encodings
    }

    /// Signature, sighash, and timelock dimensions.
    #[must_use]
    pub const fn authorization(&self) -> &AuthorizationContract {
        &self.authorization
    }

    /// Confidential-value capabilities.
    #[must_use]
    pub const fn confidential_values(&self) -> &ConfidentialValueContract {
        &self.confidential_values
    }

    /// Issuance and reissuance facts.
    #[must_use]
    pub const fn issuance(&self) -> &IssuanceContract {
        &self.issuance
    }

    /// Consensus and policy resource interfaces.
    #[must_use]
    pub const fn resources(&self) -> &ResourceContract {
        &self.resources
    }

    /// The capability registry.
    #[must_use]
    pub const fn capabilities(&self) -> &BTreeMap<ElementsCapability, CapabilityContract> {
        &self.capabilities
    }

    /// The evidence-requirement registry.
    #[must_use]
    pub const fn evidence_requirements(
        &self,
    ) -> &BTreeMap<TargetEvidenceRequirementId, TargetEvidenceRequirement> {
        &self.evidence_requirements
    }
}

/// A contract that has passed [`validate_target_definition`].
///
/// The wrapper is the only form a consumer accepts. It is opaque: a
/// caller reads the contract through [`Self::definition`] but cannot
/// construct the wrapper around a definition that has not been
/// validated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedTargetDefinition {
    definition: TargetDefinition,
}

impl ValidatedTargetDefinition {
    /// The validated contract.
    #[must_use]
    pub const fn definition(&self) -> &TargetDefinition {
        &self.definition
    }

    /// The stable semantic projection of this contract.
    #[must_use]
    pub fn projection(&self) -> TargetProjection {
        TargetProjection {
            version: self.definition.version,
            execution_domain: self.definition.execution_domain,
            leaf_version: self.definition.leaf_version,
            opcodes: self.definition.opcodes.values().cloned().collect(),
            encodings: self.definition.encodings.values().cloned().collect(),
            authorization: self.definition.authorization.clone(),
            confidential_values: self.definition.confidential_values.clone(),
            issuance: self.definition.issuance.clone(),
            resources: self.definition.resources.clone(),
            capabilities: self.definition.capabilities.values().cloned().collect(),
            evidence_requirements: self
                .definition
                .evidence_requirements
                .values()
                .cloned()
                .collect(),
        }
    }
}

/// The first-party reviewed Elements tapscript contract.
///
/// # Why this is a second wrapper rather than a flag
///
/// [`ValidatedTargetDefinition`] proves *internal coherence*: a caller
/// can assemble a locally consistent contract that permutes opcode
/// bytes, rewrites failure behavior, restates resource limits, or
/// marks its own capabilities reviewed, and the generic validator will
/// accept it, because every one of those is a well-formed contract —
/// just not this project's. A consumer whose claim is "these are the
/// Elements tapscript semantics the project reviewed" needs a stronger
/// state, and a boolean on the validated value would be settable by
/// whoever set the rest of it.
///
/// So the reviewed state is a distinct type with no public
/// constructor. The only two ways to obtain one are
/// [`reviewed_elements_tapscript`], which derives the first-party
/// contract and validates it, and [`validate_as_reviewed_elements`],
/// which admits an offered validated contract only when it is
/// *typed-equal* to that independently derived value.
///
/// The wrapper is a trust state, not a digest. Nothing here is hashed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewedElementsTapscriptDefinition {
    definition: ValidatedTargetDefinition,
}

impl ReviewedElementsTapscriptDefinition {
    /// The reviewed contract as a generic validated contract.
    #[must_use]
    pub const fn validated(&self) -> &ValidatedTargetDefinition {
        &self.definition
    }

    /// Consumes the reviewed state, yielding the validated contract.
    ///
    /// Deliberately one-way: a consumer that only needs internal
    /// coherence can drop down to the generic state, and nothing
    /// climbs back up without another exact comparison.
    #[must_use]
    pub fn into_validated(self) -> ValidatedTargetDefinition {
        self.definition
    }

    /// The reviewed contract.
    #[must_use]
    pub const fn definition(&self) -> &TargetDefinition {
        self.definition.definition()
    }

    /// The stable semantic projection of the reviewed contract.
    #[must_use]
    pub fn projection(&self) -> TargetProjection {
        self.definition.projection()
    }
}

/// The stable comparison form of a validated contract.
///
/// # What it excludes, and why that is structural
///
/// The projection carries no upstream repository, source revision,
/// source path, node version, review date, host, environment value,
/// diagnostic string, or digest. That is not enforced by a filter over
/// the projection: none of those values exists anywhere in this
/// crate's types, so there is nothing to strip. Review provenance
/// lives in the human reference and cannot reach a type here without a
/// deliberate and visible change to the type model.
///
/// The contract identifies a typed compatibility surface. Two nodes at
/// different revisions that implement the same reviewed semantics
/// satisfy the same contract, and that is the whole point of keeping
/// the revision out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetProjection {
    version: TargetContractVersion,
    execution_domain: ExecutionDomain,
    leaf_version: LeafVersion,
    opcodes: Vec<OpcodeSpec>,
    encodings: Vec<EncodingSpec>,
    authorization: AuthorizationContract,
    confidential_values: ConfidentialValueContract,
    issuance: IssuanceContract,
    resources: ResourceContract,
    capabilities: Vec<CapabilityContract>,
    evidence_requirements: Vec<TargetEvidenceRequirement>,
}

impl TargetProjection {
    /// The contract revision.
    #[must_use]
    pub const fn version(&self) -> TargetContractVersion {
        self.version
    }

    /// The domain the contract describes.
    #[must_use]
    pub const fn execution_domain(&self) -> ExecutionDomain {
        self.execution_domain
    }

    /// The leaf version the contract requires.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// The primitive contracts, in stable identity order.
    #[must_use]
    pub fn opcodes(&self) -> &[OpcodeSpec] {
        &self.opcodes
    }

    /// The encoding contracts, in stable identity order.
    #[must_use]
    pub fn encodings(&self) -> &[EncodingSpec] {
        &self.encodings
    }

    /// Signature, sighash, and timelock dimensions.
    #[must_use]
    pub const fn authorization(&self) -> &AuthorizationContract {
        &self.authorization
    }

    /// Confidential-value capabilities.
    #[must_use]
    pub const fn confidential_values(&self) -> &ConfidentialValueContract {
        &self.confidential_values
    }

    /// Issuance and reissuance facts.
    #[must_use]
    pub const fn issuance(&self) -> &IssuanceContract {
        &self.issuance
    }

    /// Consensus and policy resource interfaces.
    #[must_use]
    pub const fn resources(&self) -> &ResourceContract {
        &self.resources
    }

    /// The capability contracts, in stable identity order.
    #[must_use]
    pub fn capabilities(&self) -> &[CapabilityContract] {
        &self.capabilities
    }

    /// The evidence requirements, in stable identity order.
    #[must_use]
    pub fn evidence_requirements(&self) -> &[TargetEvidenceRequirement] {
        &self.evidence_requirements
    }
}

/// Validates an offered contract, reporting every defect found.
///
/// # Errors
///
/// Returns every [`TargetError`] the contract exhibits, in a stable
/// order. An empty error vector is impossible: the function returns
/// the validated wrapper in that case instead.
pub fn validate_target_definition(
    definition: TargetDefinition,
) -> Result<ValidatedTargetDefinition, Vec<TargetError>> {
    // The contract version and the leaf version are not re-checked
    // here, and their absence is deliberate. Neither type has a public
    // unchecked constructor: the only ways to obtain them are
    // `TargetContractVersion::supported` and `LeafVersion::new`, both
    // of which refuse an unsupported value. Re-testing them in this
    // function would add two branches no input can reach, and an
    // unreachable branch in a validator is worse than no branch at
    // all — it reads as a check that is running when it is not.
    let mut errors = Vec::new();

    validate_opcodes(&definition, &mut errors);
    validate_encodings(&definition, &mut errors);
    validate_authorization(&definition, &mut errors);
    validate_confidential_and_issuance(&definition, &mut errors);
    validate_resources(&definition, &mut errors);
    validate_capabilities(&definition, &mut errors);
    validate_evidence(&definition, &mut errors);

    if errors.is_empty() {
        Ok(ValidatedTargetDefinition { definition })
    } else {
        Err(errors)
    }
}

/// Checks the primitive registry against the reviewed contract.
fn validate_opcodes(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    // Every reviewed identity must have a contract. The census is the
    // authority: a registry missing an entry is incomplete, not a
    // registry describing a smaller target.
    for id in OpcodeId::ALL {
        if !definition.opcodes.contains_key(id) {
            errors.push(TargetError::MissingOpcodeContract(*id));
        }
    }

    let mut codes: BTreeMap<u8, OpcodeId> = BTreeMap::new();

    for (key, spec) in &definition.opcodes {
        // A map entry must not claim one identity while containing
        // another; the key would then be a lie the whole registry is
        // indexed by.
        if spec.id() != *key {
            errors.push(TargetError::OpcodeIdMismatch {
                key: *key,
                declared: spec.id(),
            });
        }

        if codes.insert(spec.code(), *key).is_some() {
            errors.push(TargetError::DuplicateOpcodeCode(spec.code()));
        }

        if spec.domains().is_empty() || !spec.domains().contains(&definition.execution_domain) {
            errors.push(TargetError::UnsupportedOpcodeExecutionDomain(*key));
        }

        if spec.stack().has_malformed_width() {
            errors.push(TargetError::InvalidOpcodeStackContract(*key));
        }

        // A primitive's successful behavior must be a relation the
        // stack arithmetic can be read off. Naming no form at all,
        // naming one form twice, consuming operands that were never
        // declared, or retaining operands there are none of each
        // leaves a consumer unable to compute a resulting depth.
        if let Some(defect) = spec.stack().success_defect() {
            errors.push(match defect {
                SuccessContractDefect::NoCases => TargetError::IncompleteSuccessContract(*key),
                SuccessContractDefect::DuplicateCondition(condition) => {
                    TargetError::DuplicateSuccessCase {
                        opcode: *key,
                        case: condition,
                    }
                }
                SuccessContractDefect::ConsumesMoreThanDeclared => {
                    TargetError::ContradictorySuccessCase(*key)
                }
                SuccessContractDefect::NothingToRetain => {
                    TargetError::InvalidRetainedOperandContract(*key)
                }
            });
        }

        if spec.stack().failure().is_empty() {
            errors.push(TargetError::MissingOpcodeFailureContract(*key));
        }

        if let Some(cause) = spec.stack().failure().contradictory_cause() {
            errors.push(TargetError::ContradictoryFailureCause {
                opcode: *key,
                cause,
            });
        }

        // Every primitive occupies at least one script byte, so a zero
        // here is an unfilled cost rather than a free operation.
        if spec.resources().script_bytes() == 0 {
            errors.push(TargetError::MissingOpcodeResourceCost(*key));
        }

        // A primitive whose semantics no deployment is ever asked to
        // demonstrate would be a claim resting on nothing but this
        // file.
        if spec.evidence().is_empty() {
            errors.push(TargetError::MissingOpcodeEvidence(*key));
        }
    }
}

/// The built-in reviewed contract for the Elements tapscript target.
///
/// This is a reviewed static contract. It is not deployment evidence,
/// it is not a production target, and it does not assert that any node
/// or network behaves as described — only that this is the behavior
/// the project has reviewed and is relying upon, so that a later
/// target-native test knows exactly what it must demonstrate.
///
/// # Errors
///
/// Returns the validator's diagnostics if the declaration in this
/// crate is itself defective. A test exercises this path, so a
/// transcription mistake fails the build rather than reaching a
/// consumer.
pub fn reviewed_elements_tapscript() -> Result<ReviewedElementsTapscriptDefinition, Vec<TargetError>>
{
    validate_target_definition(reviewed_elements_declaration())
        .map(|definition| ReviewedElementsTapscriptDefinition { definition })
}

/// The first-party reviewed declaration, before validation.
///
/// Private on purpose. Handing out the unvalidated reviewed value
/// would let a caller mutate one field and validate the result, which
/// is exactly the path [`ReviewedElementsTapscriptDefinition`] exists
/// to close.
fn reviewed_elements_declaration() -> TargetDefinition {
    TargetDefinition::new(TargetDefinitionParts {
        version: TargetContractVersion::V1,
        execution_domain: ExecutionDomain::Tapscript,
        leaf_version: LeafVersion::TAPSCRIPT,
        opcodes: reviewed_opcodes(),
        encodings: reviewed_encodings(),
        authorization: reviewed_authorization(),
        confidential_values: reviewed_confidential_values(),
        issuance: reviewed_issuance(),
        resources: reviewed_resources(),
        capabilities: reviewed_capabilities(),
        evidence_requirements: reviewed_evidence_requirements(),
    })
}

/// Promotes an offered validated contract to the reviewed state.
///
/// The offered value must be *typed-equal* to the contract this crate
/// derives independently. Nothing weaker is accepted: not the same
/// contract version, not the same leaf version, not the same opcode
/// census with different bytes. A contract that differs anywhere stays
/// generic, which is the honest description of what it is.
///
/// # Errors
///
/// Returns [`TargetError::ReviewedDefinitionMismatch`] when the
/// offered contract is not the reviewed Elements contract.
pub fn validate_as_reviewed_elements(
    offered: ValidatedTargetDefinition,
) -> Result<ReviewedElementsTapscriptDefinition, TargetError> {
    // A defective first-party declaration is reported by
    // `reviewed_elements_tapscript` and by a test that calls it; here
    // it can only mean the comparison has no reviewed value to make,
    // so the offered contract is not it.
    let expected =
        reviewed_elements_tapscript().map_err(|_| TargetError::ReviewedDefinitionMismatch)?;
    if offered == *expected.validated() {
        Ok(ReviewedElementsTapscriptDefinition {
            definition: offered,
        })
    } else {
        Err(TargetError::ReviewedDefinitionMismatch)
    }
}

/// The encoding keys the reviewed primitive registry actually depends
/// upon.
///
/// The encoding registry may describe more classes than the reviewed
/// primitives reach; this reports the subset the primitives name, so a
/// consumer can check closure in the direction that matters.
#[must_use]
pub fn encoding_dependencies(definition: &TargetDefinition) -> BTreeSet<EncodingClass> {
    use crate::opcode::StackValueType;

    let mut classes = BTreeSet::new();
    for spec in definition.opcodes.values() {
        let stack = spec.stack();
        for value in stack
            .operands()
            .iter()
            .chain(stack.success().result_types().iter())
        {
            match value {
                StackValueType::Encoded(class)
                | StackValueType::EncodedPayload(class)
                | StackValueType::EncodingPrefix(class) => {
                    classes.insert(*class);
                }
                StackValueType::EncodedPayloadAlternatives(alternatives)
                | StackValueType::EncodingPrefixAlternatives(alternatives) => {
                    classes.extend(alternatives.iter().copied());
                }
                StackValueType::Bool
                | StackValueType::ScriptNumber
                | StackValueType::Bytes { .. }
                | StackValueType::SignedFixedWidth { .. }
                | StackValueType::UnsignedFixedWidth { .. }
                | StackValueType::Empty => {}
            }
        }
    }
    classes
}

/// Checks the encoding registry.
fn validate_encodings(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    for class in EncodingClass::ALL {
        if !definition.encodings.contains_key(class) {
            errors.push(TargetError::MissingEncodingSpec(*class));
        }
    }

    // Prefixes are checked per domain rather than globally. The same
    // byte means "explicit" in the asset, value, and nonce domains
    // alike, so a global uniqueness check would reject a correct
    // contract.
    let mut claimed: BTreeMap<(crate::encoding::EncodingDomain, u8), EncodingClass> =
        BTreeMap::new();

    for (key, spec) in &definition.encodings {
        if spec.class() != *key {
            errors.push(TargetError::EncodingClassMismatch {
                key: *key,
                declared: spec.class(),
            });
        }

        if !spec.payload().is_coherent() {
            errors.push(TargetError::InvalidEncodingWidth(*key));
        }

        // A numeric payload without an order does not determine a
        // value; a non-numeric one carrying an order claims an
        // interpretation the field does not have.
        if spec.is_numeric() && spec.byte_order().is_none() {
            errors.push(TargetError::MissingByteOrder(*key));
        }
        if !spec.is_numeric() && spec.byte_order().is_some() {
            errors.push(TargetError::SpuriousByteOrder(*key));
        }

        if spec.evidence().is_empty() {
            errors.push(TargetError::MissingEncodingEvidence(*key));
        }

        for prefix in spec.prefixes() {
            if claimed.insert((spec.domain(), *prefix), *key).is_some() {
                errors.push(TargetError::DuplicateEncodingPrefix {
                    class: *key,
                    prefix: *prefix,
                });
            }
        }
    }

    // Every encoding a primitive names must exist.
    for class in encoding_dependencies(definition) {
        if !definition.encodings.contains_key(&class) {
            errors.push(TargetError::MissingEncodingSpec(class));
        }
    }
}

/// Checks the authorization contract.
fn validate_authorization(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    let sighash = definition.authorization.sighash();

    if let Some(dimension) = sighash.contradictory() {
        errors.push(TargetError::ContradictorySighashDimension(dimension));
    }
    if let Some(dimension) = sighash.unclassified() {
        errors.push(TargetError::UnclassifiedSighashDimension(dimension));
    }

    let timelock = definition.authorization.relative_timelock();
    if timelock.has_overlapping_fields() {
        errors.push(TargetError::OverlappingSequenceFields);
    }
    if timelock.modes().is_empty() {
        errors.push(TargetError::MissingTimelockMode);
    }

    for encoding in [
        definition.authorization.signature().public_key_encoding(),
        definition.authorization.signature().signature_encoding(),
    ] {
        if !definition.encodings.contains_key(&encoding) {
            errors.push(TargetError::MissingEncodingSpec(encoding));
        }
    }
}

/// Checks the confidential-value and issuance contracts.
fn validate_confidential_and_issuance(
    definition: &TargetDefinition,
    errors: &mut Vec<TargetError>,
) {
    if let Some(claim) = definition.confidential_values.unclassified() {
        errors.push(TargetError::UnclassifiedConfidentialCapability(claim));
    }

    for encoding in definition.confidential_values.participating_encodings() {
        if !definition.encodings.contains_key(encoding) {
            errors.push(TargetError::MissingEncodingSpec(*encoding));
        }
    }

    let issuance = &definition.issuance;
    if !definition.opcodes.contains_key(&issuance.introspection()) {
        errors.push(TargetError::MissingOpcodeContract(issuance.introspection()));
    }
    if !definition.encodings.contains_key(&issuance.absent_marker()) {
        errors.push(TargetError::MissingEncodingSpec(issuance.absent_marker()));
    }
}

/// Checks the resource contract.
fn validate_resources(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    let resources = &definition.resources;

    if let Some(dimension) = resources.missing_consensus_dimension() {
        errors.push(TargetError::MissingResourceDimension(dimension));
    }
    if let Some(dimension) = resources.zero_bound() {
        errors.push(TargetError::InvalidResourceContract(dimension));
    }
    if let Some(dimension) = resources.policy_looser_than_consensus() {
        errors.push(TargetError::PolicyLooserThanConsensus(dimension));
    }
    if resources.consensus().witness_scale_factor() == 0 {
        errors.push(TargetError::InvalidResourceContract(
            crate::resource::ResourceDimension::TransactionWeight,
        ));
    }
}

/// Checks the capability registry and its prerequisite structure.
fn validate_capabilities(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    for capability in ElementsCapability::ALL {
        if !definition.capabilities.contains_key(capability) {
            errors.push(TargetError::MissingCapabilityContract(*capability));
        }
    }

    for (key, contract) in &definition.capabilities {
        if contract.capability() != *key {
            errors.push(TargetError::CapabilityIdMismatch {
                key: *key,
                declared: contract.capability(),
            });
        }

        for prerequisite in contract.prerequisites() {
            if !definition.capabilities.contains_key(prerequisite) {
                errors.push(TargetError::UnknownCapabilityPrerequisite(*prerequisite));
            }
        }

        for opcode in contract.opcodes() {
            if !definition.opcodes.contains_key(opcode) {
                errors.push(TargetError::CapabilityNamesUnknownOpcode {
                    capability: *key,
                    opcode: *opcode,
                });
            }
        }

        for encoding in contract.encodings() {
            if !definition.encodings.contains_key(encoding) {
                errors.push(TargetError::CapabilityNamesUnknownEncoding {
                    capability: *key,
                    encoding: *encoding,
                });
            }
        }

        // Every capability maps to evidence, without exception. A
        // capability whose claim nothing is ever asked to demonstrate
        // rests on this crate's assertion alone, and there is no
        // capability here whose claim is purely type-level: even that
        // the execution domain exists is something a node must show.
        if contract.evidence().is_empty() {
            errors.push(TargetError::MissingCapabilityEvidence(*key));
        }
    }

    let residual = prerequisite_cycle_residual(&definition.capabilities);
    if !residual.is_empty() {
        errors.push(TargetError::CapabilityDependencyCycle { members: residual });
    }
}

/// Checks that every named evidence requirement resolves.
fn validate_evidence(definition: &TargetDefinition, errors: &mut Vec<TargetError>) {
    for id in TargetEvidenceRequirementId::ALL {
        if !definition.evidence_requirements.contains_key(id) {
            errors.push(TargetError::UnknownEvidenceRequirement(*id));
        }
    }

    for (key, requirement) in &definition.evidence_requirements {
        if requirement.id() != *key {
            errors.push(TargetError::EvidenceRequirementIdMismatch {
                key: *key,
                declared: requirement.id(),
            });
        }
        if requirement.stale_on().is_empty() {
            errors.push(TargetError::MissingStaleCondition(*key));
        }
    }

    let named = definition
        .opcodes
        .values()
        .flat_map(|spec| spec.evidence().iter().copied())
        .chain(
            definition
                .encodings
                .values()
                .flat_map(|spec| spec.evidence().iter().copied()),
        )
        .chain(
            definition
                .capabilities
                .values()
                .flat_map(|contract| contract.evidence().iter().copied()),
        );

    for id in named {
        if !definition.evidence_requirements.contains_key(&id) {
            errors.push(TargetError::UnknownEvidenceRequirement(id));
        }
    }
}
