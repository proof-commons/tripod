//! Triple recipe and exact fields are checked on the bytes being read.
use super::{EventId, Refusal, Triple};
use crate::keyed_fixed_constructor::{KeyedFixedDescriptor, PublicKeyedCurve, construct};
use realization::keyed_fixed::{KeyedFixedParameters, KeyedFixedValue};
use std::collections::{BTreeMap, BTreeSet};
use transaction::{AssetField, TargetOutput, TargetTransaction, ValueField};

pub type Recipes = BTreeMap<Triple, KeyedFixedDescriptor>;

pub fn recipes(triples: &BTreeSet<Triple>) -> Result<Recipes, Refusal> {
    let target =
        target_elements::reviewed_elements_tapscript().map_err(|_| Refusal::TargetDefinition)?;
    triples
        .iter()
        .map(|triple| {
            let value =
                KeyedFixedValue::new(triple.value).map_err(|_| Refusal::InvalidTriple(*triple))?;
            let parameters = KeyedFixedParameters {
                key: triple.key,
                asset: triple.asset,
                value,
            };
            construct(&target, parameters, &PublicKeyedCurve)
                .map(|recipe| (*triple, recipe))
                .map_err(Refusal::Recipe)
        })
        .collect()
}

pub fn program_triple(program: &[u8], recipes: &Recipes) -> Option<Triple> {
    recipes
        .iter()
        .find(|(_, recipe)| recipe.output_program == program)
        .map(|(triple, _)| *triple)
}

pub fn qualified(output: &TargetOutput, triple: Triple) -> bool {
    matches!(output.asset(), AssetField::Explicit(asset) if *asset.internal() == triple.asset)
        && output.value() == ValueField::Explicit(triple.value)
}

pub fn witness_version(bytes: &[u8]) -> Option<u8> {
    let [version, length, payload @ ..] = bytes else {
        return None;
    };
    if usize::from(*length) != payload.len() || !(2..=40).contains(&payload.len()) {
        return None;
    }
    match *version {
        0 => Some(0),
        0x51..=0x60 => Some(version - 0x50),
        _ => None,
    }
}

pub fn member(
    event: EventId,
    input: usize,
    tx: &TargetTransaction,
    recipe: &KeyedFixedDescriptor,
) -> Result<(), Refusal> {
    let successor = tx
        .outputs()
        .first()
        .ok_or(Refusal::WrongSuccessor { event, input })?;
    if witness_version(successor.program()).is_some_and(|version| version != 1) {
        return Err(Refusal::WrongVersion { event, input });
    }
    if successor.program() != recipe.output_program {
        return Err(Refusal::WrongSuccessor { event, input });
    }
    let triple = Triple {
        key: recipe.parameters.key,
        asset: recipe.parameters.asset,
        value: recipe.parameters.value.get(),
    };
    if !qualified(successor, triple) {
        return Err(Refusal::WrongSuccessorPair { event, input });
    }
    if !tx.inputs()[input].script_sig().is_empty() {
        return Err(Refusal::ScriptSig { event, input });
    }
    let stack = tx
        .witnesses()
        .get(input)
        .map(transaction::InputWitness::stack)
        .unwrap_or_default();
    // Consensus removes a final 0x50-tagged annex before script-path checks.
    let stack = if stack.len() >= 2 && stack.last().is_some_and(|item| item.first() == Some(&0x50))
    {
        &stack[..stack.len() - 1]
    } else {
        stack
    };
    if stack.len() != 2 {
        return Err(Refusal::WitnessShape { event, input });
    }
    if stack[0] != recipe.leaf {
        return Err(Refusal::UncommittedLeaf { event, input });
    }
    let control = &stack[1];
    if control
        .first()
        .is_some_and(|byte| byte & 0xfe != recipe.leaf_version)
    {
        return Err(Refusal::ControlVersion { event, input });
    }
    if control
        .first()
        .is_some_and(|byte| byte & 1 != u8::from(recipe.parity))
    {
        return Err(Refusal::WrongParity { event, input });
    }
    if *control != recipe.control {
        return Err(Refusal::ControlRecipe { event, input });
    }
    Ok(())
}
