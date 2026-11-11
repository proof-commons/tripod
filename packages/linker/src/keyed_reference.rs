//! Constructor offers and the closed occurrence inventory of the keyed leaf.
//!
//! Offers select no program. Recipe fields are equality-checked against sealed
//! emission material; introspection can never be replaced with a literal.

use tapscript::keyed_constructor::KeyedLeafRecipe;
use target_elements::ReviewedElementsTapscriptDefinition;

/// Exactly-once constructor fields, separate from instruction relocations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum KeyedConstructorField {
    Recipe,
    TargetVersion,
}

/// Authenticated execution reads, including the selectors that feed them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum KeyedReference {
    BirthCurrentInput,
    ConsumedOutpoint,
    ContinuationCurrentInput,
    CurrentProgram,
    SuccessorProgram,
}

/// Offered field identity. Unknown extensions are refused, never ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum KeyedParameter {
    Constructor(KeyedConstructorField),
    /// A mistaken attempt to resolve an execution read at link time.
    Introspection(KeyedReference),
    /// Reserved offers have no admitted meaning in this schema.
    Extension(u16),
}

/// Equality assertions; no script, comparison program or hash is an offer.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedParameterValue {
    Recipe(KeyedLeafRecipe),
    TargetVersion(u32),
}

/// Untrusted constructor offer, checked by the sole linked-bundle producer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedParameterOffer {
    pub parameter: KeyedParameter,
    pub value: KeyedParameterValue,
}

/// Canonical recipe equality assertion and target-version check.
///
/// Echoing the emission's recipe asserts nothing. A request consumer builds the
/// recipe independently from its request key, never from the emission. Schema 1
/// in `tapscript::keyed_constructor::KeyedLeafRecipe` alone specifies commitment;
/// these offers neither implement nor authenticate those construction rules.
#[must_use]
pub fn keyed_parameter_offers(
    target: &ReviewedElementsTapscriptDefinition,
    recipe: KeyedLeafRecipe,
) -> Vec<KeyedParameterOffer> {
    use KeyedConstructorField as F;
    use KeyedParameterValue as V;
    [
        (F::Recipe, V::Recipe(recipe)),
        (
            F::TargetVersion,
            V::TargetVersion(target.definition().version().get()),
        ),
    ]
    .into_iter()
    .map(|(field, value)| KeyedParameterOffer {
        parameter: KeyedParameter::Constructor(field),
        value,
    })
    .collect()
}

/// A fixed push already present in emission, never a relocation target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedLiteralKind {
    SemanticKey,
    BornOutputIndex,
    SuccessorOutputIndex,
    FinalTruth,
}

/// A readable occurrence; editing a copy cannot alter the sealed link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedLiteralOccurrence {
    pub instruction: usize,
    pub kind: KeyedLiteralKind,
    pub bytes: Vec<u8>,
}

/// One retained execution reference at a typed instruction position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedReferenceOccurrence {
    pub instruction: usize,
    pub reference: KeyedReference,
}

/// Read-only closure owned by the linked bundle, not caller-supplied authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedLinkClosure {
    pub(crate) constructors: Vec<KeyedParameterOffer>,
    pub(crate) literals: Vec<KeyedLiteralOccurrence>,
    pub(crate) references: Vec<KeyedReferenceOccurrence>,
}

impl KeyedLinkClosure {
    /// Each constructor field occurs once, in canonical field order.
    #[must_use]
    pub fn constructors(&self) -> &[KeyedParameterOffer] {
        &self.constructors
    }
    /// All four literal occurrences, copied from the checked instructions.
    #[must_use]
    pub fn literals(&self) -> &[KeyedLiteralOccurrence] {
        &self.literals
    }
    /// All five authenticated-read/selector occurrences.
    #[must_use]
    pub fn references(&self) -> &[KeyedReferenceOccurrence] {
        &self.references
    }
}
