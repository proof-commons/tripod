//! Fixed-form streaming schedule from docs/lineage/realization.md.

use target_elements::{OpcodeId, ReviewedElementsTapscriptDefinition};

use crate::keyed_fixed_constructor::KeyedFixedLeafRecipe;
use crate::{StackItem, TapscriptError, TapscriptInstruction, TapscriptProgram};

pub fn compose(
    target: &ReviewedElementsTapscriptDefinition,
    recipe: KeyedFixedLeafRecipe,
) -> Result<TapscriptProgram, TapscriptError> {
    use TapscriptInstruction::{Opcode, Push};
    let mut instructions = vec![
        Push(StackItem::new(target, recipe.parameters().key.to_vec())?),
        Opcode(OpcodeId::Drop),
        Push(StackItem::new(target, recipe.pair_digest().to_vec())?),
        Opcode(OpcodeId::Duplicate),
    ];
    instructions.extend(pair_check(target, false)?.instructions().iter().cloned());
    instructions.extend(pair_check(target, true)?.instructions().iter().cloned());
    instructions.extend(
        crate::keyed_pattern::born_first_fragment_untraced(target)?
            .instructions()
            .iter()
            .cloned(),
    );
    instructions.extend(
        crate::keyed_pattern::continue_first_fragment_untraced(target)?
            .instructions()
            .iter()
            .cloned(),
    );
    instructions.push(Push(StackItem::script_number(target, 1)?));
    TapscriptProgram::new(instructions)
}

// Consumes one constant digest. Introspection supplies payload before prefix.
pub fn qualification(
    target: &ReviewedElementsTapscriptDefinition,
    recipe: KeyedFixedLeafRecipe,
    input: bool,
) -> Result<TapscriptProgram, TapscriptError> {
    let mut instructions = vec![TapscriptInstruction::Push(StackItem::new(
        target,
        recipe.pair_digest().to_vec(),
    )?)];
    instructions.extend(pair_check(target, input)?.instructions().iter().cloned());
    if !input {
        instructions.extend(
            crate::keyed_pattern::continue_first_fragment_untraced(target)?
                .instructions()
                .iter()
                .cloned(),
        );
    }
    TapscriptProgram::new(instructions)
}

// Consumes one constant digest. Introspection supplies payload before prefix.
fn pair_check(
    target: &ReviewedElementsTapscriptDefinition,
    input: bool,
) -> Result<TapscriptProgram, TapscriptError> {
    use OpcodeId as O;
    use TapscriptInstruction::{Opcode, Push};
    let selector = if input {
        Opcode(O::PushCurrentInputIndex)
    } else {
        Push(StackItem::script_number(target, 0)?)
    };
    TapscriptProgram::new(vec![
        selector.clone(),
        Opcode(if input {
            O::InspectInputAsset
        } else {
            O::InspectOutputAsset
        }),
        Opcode(O::Concatenate),
        Opcode(O::Sha256Initialize),
        selector,
        Opcode(if input {
            O::InspectInputValue
        } else {
            O::InspectOutputValue
        }),
        Opcode(O::Concatenate),
        Opcode(O::Sha256Finalize),
        Opcode(O::EqualVerify),
    ])
}
