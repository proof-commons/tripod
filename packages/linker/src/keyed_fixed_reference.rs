//! Constructor offers and the closed occurrence inventory of the keyed leaf.
//!
//! Offers select no program. Recipe fields are equality-checked against sealed
//! emission material; introspection can never be replaced with a literal.

use tapscript::keyed_fixed_constructor::KeyedFixedLeafRecipe;
use target_elements::ReviewedElementsTapscriptDefinition;

/// Exactly-once constructor fields, separate from instruction relocations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum KeyedFixedConstructorField {
    Recipe,
    TargetVersion,
}

/// Authenticated execution reads, including the selectors that feed them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum KeyedFixedReference {
    OwnAssetCurrentInput,
    OwnAsset,
    OwnValueCurrentInput,
    OwnValue,
    SuccessorAsset,
    SuccessorValue,
    BirthCurrentInput,
    ConsumedOutpoint,
    ContinuationCurrentInput,
    CurrentProgram,
    SuccessorProgram,
}

/// Offered field identity. Unknown extensions are refused, never ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum KeyedFixedParameter {
    Constructor(KeyedFixedConstructorField),
    /// A mistaken attempt to resolve an execution read at link time.
    Introspection(KeyedFixedReference),
    /// Reserved offers have no admitted meaning in this schema.
    Extension(u16),
}

/// Equality assertions; no script, comparison program or hash is an offer.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedFixedParameterValue {
    Recipe(KeyedFixedLeafRecipe),
    TargetVersion(u32),
}

/// Untrusted constructor offer, checked by the sole linked-bundle producer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedFixedParameterOffer {
    pub parameter: KeyedFixedParameter,
    pub value: KeyedFixedParameterValue,
}

/// Canonical recipe equality assertion and target-version check.
///
/// Echoing the emission's recipe asserts nothing. A request consumer builds the
/// recipe independently from its request triple, never from the emission. Schema 1
/// in `tapscript::keyed_fixed_constructor::KeyedFixedLeafRecipe` alone specifies commitment;
/// these offers neither implement nor authenticate those construction rules.
#[must_use]
pub fn keyed_fixed_parameter_offers(
    target: &ReviewedElementsTapscriptDefinition,
    recipe: KeyedFixedLeafRecipe,
) -> Vec<KeyedFixedParameterOffer> {
    use KeyedFixedConstructorField as F;
    use KeyedFixedParameterValue as V;
    [
        (F::Recipe, V::Recipe(recipe)),
        (
            F::TargetVersion,
            V::TargetVersion(target.definition().version().get()),
        ),
    ]
    .into_iter()
    .map(|(field, value)| KeyedFixedParameterOffer {
        parameter: KeyedFixedParameter::Constructor(field),
        value,
    })
    .collect()
}

/// A fixed push already present in emission, never a relocation target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedFixedLiteralKind {
    SemanticKey,
    PairDigest,
    SuccessorAssetIndex,
    SuccessorValueIndex,
    BornOutputIndex,
    SuccessorOutputIndex,
    FinalTruth,
}

/// A readable occurrence; editing a copy cannot alter the sealed link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedFixedLiteralOccurrence {
    pub instruction: usize,
    pub kind: KeyedFixedLiteralKind,
    pub bytes: Vec<u8>,
}

/// One retained execution reference at a typed instruction position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedFixedReferenceOccurrence {
    pub instruction: usize,
    pub reference: KeyedFixedReference,
}

/// Read-only closure owned by the linked bundle, not caller-supplied authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedFixedLinkClosure {
    pub(crate) constructors: Vec<KeyedFixedParameterOffer>,
    pub(crate) literals: Vec<KeyedFixedLiteralOccurrence>,
    pub(crate) references: Vec<KeyedFixedReferenceOccurrence>,
}

impl KeyedFixedLinkClosure {
    /// Each constructor field occurs once, in canonical field order.
    #[must_use]
    pub fn constructors(&self) -> &[KeyedFixedParameterOffer] {
        &self.constructors
    }
    /// All seven literal occurrences, copied from the checked instructions.
    #[must_use]
    pub fn literals(&self) -> &[KeyedFixedLiteralOccurrence] {
        &self.literals
    }
    /// All eleven authenticated-read/selector occurrences.
    #[must_use]
    pub fn references(&self) -> &[KeyedFixedReferenceOccurrence] {
        &self.references
    }
}
