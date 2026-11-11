//! Exact closure of sealed keyed emission, without planning or commitment.

use crate::keyed_reference::{
    KeyedConstructorField, KeyedLinkClosure, KeyedLiteralKind, KeyedLiteralOccurrence,
    KeyedParameter, KeyedParameterOffer, KeyedReference, KeyedReferenceOccurrence,
    keyed_parameter_offers,
};
use std::collections::BTreeSet;
use tapscript::{
    TapscriptInstruction,
    keyed_bundle::{KeyedBundleProjection, KeyedEmissionBundle},
};
use target_elements::{OpcodeId, ReviewedElementsTapscriptDefinition};

/// Immutable link. The only producer checks constructor and occurrence closure.
///
/// ```compile_fail
/// use linker::keyed_bundle::LinkedKeyedBundle;
/// fn forge() -> LinkedKeyedBundle { LinkedKeyedBundle {} }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedKeyedBundle {
    emission: KeyedEmissionBundle,
    closure: KeyedLinkClosure,
    bytes: Vec<u8>,
}

impl LinkedKeyedBundle {
    /// The retained sealed emission for downstream member binding.
    #[must_use]
    pub const fn emission(&self) -> &KeyedEmissionBundle {
        &self.emission
    }

    /// Retains binding, scope, selected proof/placements, coverage and evidence.
    #[must_use]
    pub const fn project(&self) -> &KeyedBundleProjection {
        self.emission.project()
    }
    /// Checked constructor, literal and execution-reference inventories.
    #[must_use]
    pub const fn closure(&self) -> &KeyedLinkClosure {
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
pub enum KeyedLinkRefusal {
    MissingConstructor(KeyedConstructorField),
    DuplicateConstructor(KeyedConstructorField),
    UnknownParameter(u16),
    IntrospectionSubstitution(KeyedReference),
    TargetMismatch,
    RecipeMismatch,
    /// Internal sealed-material consistency guard.
    InstructionMismatch {
        instruction: usize,
    },
}

impl KeyedLinkRefusal {
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
pub fn link_keyed_bundle(
    target: &ReviewedElementsTapscriptDefinition,
    emission: &KeyedEmissionBundle,
    offers: &[KeyedParameterOffer],
) -> Result<LinkedKeyedBundle, KeyedLinkRefusal> {
    let _span = tracing::debug_span!(target: "linker::keyed_bundle", "link_keyed_bundle",
        family = "keyed", offer_count = count(offers.len()))
    .entered();
    let result = link(target, emission, offers);
    match &result {
        Ok(bundle) => tracing::debug!(target: "linker::keyed_bundle", family = "keyed",
            outcome = "completed", constructor_count = count(bundle.closure.constructors.len()),
            literal_count = count(bundle.closure.literals.len()), reference_count = count(bundle.closure.references.len()),
            relocation_count = 0_u64, leaf_count = 1_u64,
            outstanding_obligation_count = count(bundle.project().evidence.len()), "keyed link completed"),
        Err(error) => tracing::debug!(target: "linker::keyed_bundle", family = "keyed",
            outcome = "refused", refusal_kind = error.refusal_kind(), "keyed link refused"),
    }
    result
}

fn count(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

fn link(
    target: &ReviewedElementsTapscriptDefinition,
    emission: &KeyedEmissionBundle,
    offers: &[KeyedParameterOffer],
) -> Result<LinkedKeyedBundle, KeyedLinkRefusal> {
    let material = &emission.project().successor;
    let constructors = keyed_parameter_offers(target, *material.recipe());
    check_offers(&constructors, offers)?;
    let mut closure = inventory(material.program().instructions(), material.recipe().key())?;
    closure.constructors = constructors;
    Ok(LinkedKeyedBundle {
        emission: emission.clone(),
        closure,
        bytes: material.program().encode(target),
    })
}

fn check_offers(
    expected: &[KeyedParameterOffer],
    offers: &[KeyedParameterOffer],
) -> Result<(), KeyedLinkRefusal> {
    let mut seen = BTreeSet::new();
    for offer in offers {
        let field = match offer.parameter {
            KeyedParameter::Constructor(field) => field,
            KeyedParameter::Introspection(reference) => {
                return Err(KeyedLinkRefusal::IntrospectionSubstitution(reference));
            }
            KeyedParameter::Extension(code) => {
                return Err(KeyedLinkRefusal::UnknownParameter(code));
            }
        };
        if !seen.insert(field) {
            return Err(KeyedLinkRefusal::DuplicateConstructor(field));
        }
        let original = expected
            .iter()
            .find(|row| row.parameter == offer.parameter)
            .ok_or(match field {
                KeyedConstructorField::Recipe => KeyedLinkRefusal::RecipeMismatch,
                KeyedConstructorField::TargetVersion => KeyedLinkRefusal::TargetMismatch,
            })?;
        if original.value != offer.value {
            return Err(match field {
                KeyedConstructorField::TargetVersion => KeyedLinkRefusal::TargetMismatch,
                KeyedConstructorField::Recipe => KeyedLinkRefusal::RecipeMismatch,
            });
        }
    }
    for row in expected {
        if let KeyedParameter::Constructor(field) = row.parameter
            && !seen.contains(&field)
        {
            return Err(KeyedLinkRefusal::MissingConstructor(field));
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
) -> Result<KeyedLinkClosure, KeyedLinkRefusal> {
    use KeyedLiteralKind as L;
    use KeyedReference as R;
    use OpcodeId as O;
    let mut closure = KeyedLinkClosure {
        constructors: Vec::new(),
        literals: Vec::new(),
        references: Vec::new(),
    };
    if instructions.len() != 16 {
        return Err(KeyedLinkRefusal::InstructionMismatch {
            instruction: instructions.len().min(16),
        });
    }
    for (i, instruction) in instructions.iter().enumerate() {
        let literal = match i {
            0 => Some((L::SemanticKey, key.as_slice())),
            5 => Some((L::BornOutputIndex, &[0, 0, 0, 0][..])),
            10 => Some((L::SuccessorOutputIndex, &[][..])),
            15 => Some((L::FinalTruth, &[1][..])),
            _ => None,
        };
        if let Some((kind, bytes)) = literal {
            match instruction {
                TapscriptInstruction::Push(item) if item.bytes() == bytes => {
                    closure.literals.push(KeyedLiteralOccurrence {
                        instruction: i,
                        kind,
                        bytes: item.bytes().to_vec(),
                    });
                }
                _ => return Err(KeyedLinkRefusal::InstructionMismatch { instruction: i }),
            }
            continue;
        }
        let (opcode, reference) = match i {
            1 | 4 | 7 => (O::Drop, None),
            2 => (O::PushCurrentInputIndex, Some(R::BirthCurrentInput)),
            3 => (O::InspectInputOutpoint, Some(R::ConsumedOutpoint)),
            6 | 13 | 14 => (O::EqualVerify, None),
            8 => (O::PushCurrentInputIndex, Some(R::ContinuationCurrentInput)),
            9 => (O::InspectInputScriptPubKey, Some(R::CurrentProgram)),
            11 => (O::InspectOutputScriptPubKey, Some(R::SuccessorProgram)),
            12 => (O::Rotate, None),
            _ => return Err(KeyedLinkRefusal::InstructionMismatch { instruction: i }),
        };
        if instruction != &TapscriptInstruction::Opcode(opcode) {
            return Err(KeyedLinkRefusal::InstructionMismatch { instruction: i });
        }
        if let Some(reference) = reference {
            closure.references.push(KeyedReferenceOccurrence {
                instruction: i,
                reference,
            });
        }
    }
    Ok(closure)
}

#[cfg(test)]
mod tests;
