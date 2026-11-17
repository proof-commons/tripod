//! Exact closure of sealed keyed emission, without planning or commitment.

use crate::keyed_fixed_reference::{
    KeyedFixedConstructorField, KeyedFixedLinkClosure, KeyedFixedLiteralKind,
    KeyedFixedLiteralOccurrence, KeyedFixedParameter, KeyedFixedParameterOffer,
    KeyedFixedReference, KeyedFixedReferenceOccurrence, keyed_fixed_parameter_offers,
};
use std::collections::BTreeSet;
use tapscript::{
    TapscriptInstruction,
    keyed_fixed_bundle::{KeyedFixedBundleProjection, KeyedFixedEmissionBundle},
};
use target_elements::{OpcodeId, ReviewedElementsTapscriptDefinition};

/// Immutable link. The only producer checks constructor and occurrence closure.
///
/// ```compile_fail,E0451
/// use linker::{keyed_fixed_bundle::LinkedKeyedFixedBundle, keyed_fixed_reference::KeyedFixedLinkClosure};
/// use tapscript::keyed_fixed_bundle::KeyedFixedEmissionBundle;
/// fn forge(emission: KeyedFixedEmissionBundle, closure: KeyedFixedLinkClosure) -> LinkedKeyedFixedBundle {
///     LinkedKeyedFixedBundle { emission, closure, bytes: Vec::new() }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedKeyedFixedBundle {
    emission: KeyedFixedEmissionBundle,
    closure: KeyedFixedLinkClosure,
    bytes: Vec<u8>,
}

impl LinkedKeyedFixedBundle {
    /// The retained sealed emission for downstream member binding.
    #[must_use]
    pub const fn emission(&self) -> &KeyedFixedEmissionBundle {
        &self.emission
    }

    /// Retains binding, scope, selected proof/placements, coverage and evidence.
    #[must_use]
    pub const fn project(&self) -> &KeyedFixedBundleProjection {
        self.emission.project()
    }
    /// Checked constructor, literal and execution-reference inventories.
    #[must_use]
    pub const fn closure(&self) -> &KeyedFixedLinkClosure {
        &self.closure
    }
    /// Unchanged encoded leaf. No mutable access to linked bytes exists.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Link refusals, never a covenant execution or policy-admission verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedFixedLinkRefusal {
    MissingConstructor(KeyedFixedConstructorField),
    DuplicateConstructor(KeyedFixedConstructorField),
    UnknownParameter(u16),
    IntrospectionSubstitution(KeyedFixedReference),
    TargetMismatch,
    RecipeMismatch,
    /// Internal sealed-material consistency guard.
    InstructionMismatch {
        instruction: usize,
    },
}

impl KeyedFixedLinkRefusal {
    /// Payload-free ADR-026 classification.
    #[must_use]
    pub const fn refusal_kind(&self) -> &'static str {
        match self {
            Self::MissingConstructor(_) => "missing_constructor",
            Self::DuplicateConstructor(_) => "duplicate_constructor",
            Self::UnknownParameter(_) => "unknown_parameter",
            Self::IntrospectionSubstitution(_) => "introspection_substitution",
            Self::TargetMismatch => "target_mismatch",
            Self::RecipeMismatch => "recipe_mismatch",
            Self::InstructionMismatch { .. } => "instruction_mismatch",
        }
    }
}

/// Link one sealed emission with exactly one recipe assertion and target check.
///
/// The recipe offer is an equality assertion: echoing emission asserts nothing.
/// Request consumers build it independently from their request key. Commitment
/// follows tapscript's schema 1, not linker constants.
/// No funding, curve arithmetic, evidence admission or compiler search occurs.
/// # Errors
/// Refuses missing, duplicate, unknown or mismatched offers and any internal
/// material/reference inconsistency. Execution references cannot be substituted.
pub fn link_keyed_fixed_bundle(
    target: &ReviewedElementsTapscriptDefinition,
    emission: &KeyedFixedEmissionBundle,
    offers: &[KeyedFixedParameterOffer],
) -> Result<LinkedKeyedFixedBundle, KeyedFixedLinkRefusal> {
    let _span =
        tracing::debug_span!(target: "linker::keyed_fixed_bundle", "link_keyed_fixed_bundle",
        family = "keyed", offer_count = count(offers.len()))
        .entered();
    let result = link(target, emission, offers);
    match &result {
        Ok(bundle) => tracing::debug!(target: "linker::keyed_fixed_bundle", family = "keyed",
            outcome = "completed", constructor_count = count(bundle.closure.constructors.len()),
            literal_count = count(bundle.closure.literals.len()), reference_count = count(bundle.closure.references.len()),
            relocation_count = 0_u64, leaf_count = 1_u64,
            outstanding_obligation_count = count(bundle.project().evidence.len()), "keyed link completed"),
        Err(error) => tracing::debug!(target: "linker::keyed_fixed_bundle", family = "keyed",
            outcome = "refused", refusal_kind = error.refusal_kind(), "keyed link refused"),
    }
    result
}

fn count(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

fn link(
    target: &ReviewedElementsTapscriptDefinition,
    emission: &KeyedFixedEmissionBundle,
    offers: &[KeyedFixedParameterOffer],
) -> Result<LinkedKeyedFixedBundle, KeyedFixedLinkRefusal> {
    let material = &emission.project().successor;
    let constructors = keyed_fixed_parameter_offers(target, *material.recipe());
    check_offers(&constructors, offers)?;
    let mut closure = inventory(
        material.program().instructions(),
        &material.recipe().parameters().key,
        &material.recipe().pair_digest(),
    )?;
    closure.constructors = constructors;
    Ok(LinkedKeyedFixedBundle {
        emission: emission.clone(),
        closure,
        bytes: material.program().encode(target),
    })
}

fn check_offers(
    expected: &[KeyedFixedParameterOffer],
    offers: &[KeyedFixedParameterOffer],
) -> Result<(), KeyedFixedLinkRefusal> {
    let mut seen = BTreeSet::new();
    for offer in offers {
        let field = match offer.parameter {
            KeyedFixedParameter::Constructor(field) => field,
            KeyedFixedParameter::Introspection(reference) => {
                return Err(KeyedFixedLinkRefusal::IntrospectionSubstitution(reference));
            }
            KeyedFixedParameter::Extension(code) => {
                return Err(KeyedFixedLinkRefusal::UnknownParameter(code));
            }
        };
        if !seen.insert(field) {
            return Err(KeyedFixedLinkRefusal::DuplicateConstructor(field));
        }
        let original = expected
            .iter()
            .find(|row| row.parameter == offer.parameter)
            .ok_or(match field {
                KeyedFixedConstructorField::Recipe => KeyedFixedLinkRefusal::RecipeMismatch,
                KeyedFixedConstructorField::TargetVersion => KeyedFixedLinkRefusal::TargetMismatch,
            })?;
        if original.value != offer.value {
            return Err(match field {
                KeyedFixedConstructorField::TargetVersion => KeyedFixedLinkRefusal::TargetMismatch,
                KeyedFixedConstructorField::Recipe => KeyedFixedLinkRefusal::RecipeMismatch,
            });
        }
    }
    for row in expected {
        if let KeyedFixedParameter::Constructor(field) = row.parameter
            && !seen.contains(&field)
        {
            return Err(KeyedFixedLinkRefusal::MissingConstructor(field));
        }
    }
    Ok(())
}

// This is an occurrence checker, not another emitter or abstract evaluator.
// Every instruction must be accounted for, so an unrecognized symbolic site
// cannot silently acquire a zero-relocation report.
fn inventory(
    instructions: &[TapscriptInstruction],
    key: &[u8; 32],
    digest: &[u8; 32],
) -> Result<KeyedFixedLinkClosure, KeyedFixedLinkRefusal> {
    use KeyedFixedLiteralKind as L;
    use KeyedFixedReference as R;
    use OpcodeId as O;
    let mut closure = KeyedFixedLinkClosure {
        constructors: Vec::new(),
        literals: Vec::new(),
        references: Vec::new(),
    };
    if instructions.len() != 36 {
        return Err(KeyedFixedLinkRefusal::InstructionMismatch {
            instruction: instructions.len().min(36),
        });
    }
    for (i, instruction) in instructions.iter().enumerate() {
        let literal = match i {
            0 => Some((L::SemanticKey, key.as_slice())),
            2 => Some((L::PairDigest, digest.as_slice())),
            4 => Some((L::SuccessorAssetIndex, &[][..])),
            8 => Some((L::SuccessorValueIndex, &[][..])),
            25 => Some((L::BornOutputIndex, &[0, 0, 0, 0][..])),
            30 => Some((L::SuccessorOutputIndex, &[][..])),
            35 => Some((L::FinalTruth, &[1][..])),
            _ => None,
        };
        if let Some((kind, bytes)) = literal {
            match instruction {
                TapscriptInstruction::Push(item) if item.bytes() == bytes => {
                    closure.literals.push(KeyedFixedLiteralOccurrence {
                        instruction: i,
                        kind,
                        bytes: item.bytes().to_vec(),
                    });
                }
                _ => return Err(KeyedFixedLinkRefusal::InstructionMismatch { instruction: i }),
            }
            continue;
        }
        let (opcode, reference) = match i {
            1 | 24 | 27 => (O::Drop, None),
            3 => (O::Duplicate, None),
            5 => (O::InspectOutputAsset, Some(R::SuccessorAsset)),
            6 | 10 | 15 | 19 => (O::Concatenate, None),
            7 | 16 => (O::Sha256Initialize, None),
            9 => (O::InspectOutputValue, Some(R::SuccessorValue)),
            11 | 20 => (O::Sha256Finalize, None),
            12 | 21 | 26 | 33 | 34 => (O::EqualVerify, None),
            13 => (O::PushCurrentInputIndex, Some(R::OwnAssetCurrentInput)),
            14 => (O::InspectInputAsset, Some(R::OwnAsset)),
            17 => (O::PushCurrentInputIndex, Some(R::OwnValueCurrentInput)),
            18 => (O::InspectInputValue, Some(R::OwnValue)),
            22 => (O::PushCurrentInputIndex, Some(R::BirthCurrentInput)),
            23 => (O::InspectInputOutpoint, Some(R::ConsumedOutpoint)),
            28 => (O::PushCurrentInputIndex, Some(R::ContinuationCurrentInput)),
            29 => (O::InspectInputScriptPubKey, Some(R::CurrentProgram)),
            31 => (O::InspectOutputScriptPubKey, Some(R::SuccessorProgram)),
            32 => (O::Rotate, None),
            _ => return Err(KeyedFixedLinkRefusal::InstructionMismatch { instruction: i }),
        };
        if instruction != &TapscriptInstruction::Opcode(opcode) {
            return Err(KeyedFixedLinkRefusal::InstructionMismatch { instruction: i });
        }
        if let Some(reference) = reference {
            closure.references.push(KeyedFixedReferenceOccurrence {
                instruction: i,
                reference,
            });
        }
    }
    Ok(closure)
}
