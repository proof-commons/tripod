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

use crate::encoding::EncodingClass;
use crate::error::TargetError;
use crate::opcode::{ExecutionDomain, LeafVersion, OpcodeId, OpcodeSpec, reviewed_opcodes};

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
}

impl TargetDefinition {
    /// Assembles an unvalidated contract.
    ///
    /// The value this returns carries no guarantee whatever. It is an
    /// input to the validator, and nothing downstream accepts it.
    #[must_use]
    pub const fn new(
        version: TargetContractVersion,
        execution_domain: ExecutionDomain,
        leaf_version: LeafVersion,
        opcodes: BTreeMap<OpcodeId, OpcodeSpec>,
    ) -> Self {
        Self {
            version,
            execution_domain,
            leaf_version,
            opcodes,
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
        }
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
pub fn reviewed_elements_tapscript() -> Result<ValidatedTargetDefinition, Vec<TargetError>> {
    validate_target_definition(TargetDefinition::new(
        TargetContractVersion::V1,
        ExecutionDomain::Tapscript,
        LeafVersion::TAPSCRIPT,
        reviewed_opcodes(),
    ))
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
            .chain(stack.success_results().iter())
        {
            match value {
                StackValueType::Encoded(class)
                | StackValueType::EncodedPayload(class)
                | StackValueType::EncodingPrefix(class) => {
                    classes.insert(*class);
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
