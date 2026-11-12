//! Shared substrate only: public recipe, conformance curve and bounded byte core.
//! None of these operations supplies a recurrence answer.
use super::{Address, EventId, Key, LeafEvidence, Member, Refusal, Relation, RelationEvidence};
use crate::keyed_constructor::KeyedConstructorDescriptor;
use crate::keyed_constructor::PublicKeyedCurve;
use crate::keyed_constructor::construct_keyed;
use crate::keyed_stack_core as core;
use crate::keyed_stack_core::Facts;
use crate::keyed_stack_core::InputFact;
use crate::keyed_stack_core::ProgramFact;
use std::collections::{BTreeMap, BTreeSet};
use target_elements_conformance::constructor::tagged::sha256;
use transaction::TargetTransaction;

pub type Recipes = BTreeMap<Key, KeyedConstructorDescriptor>;

pub fn recipes(keys: &BTreeSet<Key>) -> Result<Recipes, Refusal> {
    let target =
        target_elements::reviewed_elements_tapscript().map_err(|_| Refusal::TargetDefinition)?;
    keys.iter()
        .map(|key| {
            construct_keyed(
                &target,
                tapscript::keyed_constructor::KeyedSemanticKey::new(key.0),
                &PublicKeyedCurve,
            )
            .map(|recipe| (*key, recipe))
            .map_err(Refusal::Recipe)
        })
        .collect()
}

pub fn key(program: &[u8], recipes: &Recipes) -> Option<Key> {
    recipes
        .iter()
        .find(|(_, recipe)| recipe.output_program == program)
        .map(|(key, _)| *key)
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
fn program(bytes: &[u8]) -> ProgramFact {
    witness_version(bytes).map_or_else(
        || ProgramFact::Other {
            sha256: sha256(bytes),
        },
        |version| ProgramFact::Witness {
            version,
            payload: bytes[2..].to_vec(),
        },
    )
}

pub fn context(index: &super::identity::Index, tx: &TargetTransaction) -> Facts {
    Facts {
        enabled: true,
        current: None,
        inputs: tx
            .inputs()
            .iter()
            .map(|row| {
                let point = row.outpoint();
                let previous = index
                    .parents
                    .get(&Address(*point.txid().internal()))
                    .and_then(|tx| tx.outputs().get(usize::try_from(point.index()).ok()?));
                InputFact {
                    txid: *point.txid().internal(),
                    index: point.index(),
                    flags: 0,
                    program: previous
                        .map_or(ProgramFact::Unavailable, |output| program(output.program())),
                }
            })
            .collect(),
        outputs: tx
            .outputs()
            .iter()
            .map(|output| program(output.program()))
            .collect(),
    }
}

pub fn member(
    event: EventId,
    input: usize,
    member: Member,
    tx: &TargetTransaction,
    recipe: &KeyedConstructorDescriptor,
    facts: &Facts,
) -> Result<([RelationEvidence; 2], LeafEvidence), Refusal> {
    if member.position != 0 {
        return Err(Refusal::LaterBirth {
            event,
            input,
            address: member.address,
            position: member.position,
        });
    }
    let successor = tx
        .outputs()
        .first()
        .map(transaction::TargetOutput::program)
        .unwrap_or_default();
    if witness_version(successor).is_some_and(|version| version != 1) {
        return Err(Refusal::WrongVersion { event, input });
    }
    if successor != recipe.output_program {
        return Err(Refusal::WrongSuccessor { event, input });
    }
    if !tx.inputs()[input].script_sig().is_empty() {
        return Err(Refusal::ScriptSig { event, input });
    }
    let stack = tx
        .witnesses()
        .get(input)
        .map(transaction::InputWitness::stack)
        .unwrap_or_default();
    if stack.len() != 2 {
        return Err(Refusal::WitnessShape { event, input });
    }
    if stack[0] != recipe.leaf_bytes {
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
        .is_some_and(|byte| byte & 1 != u8::from(recipe.control.parity))
    {
        return Err(Refusal::WrongParity { event, input });
    }
    if *control != recipe.control.bytes {
        return Err(Refusal::ControlRecipe { event, input });
    }
    let mut facts = facts.clone();
    facts.current = Some(u32::try_from(input).map_err(|_| Refusal::Model {
        event,
        input,
        kind: "input_index",
    })?);
    // Schema-one grammar: key/drop occupies 34 bytes, birth 10, continuation 7.
    // Authentication above fixes the complete leaf before any slice is evaluated.
    let traces = [&stack[0][34..44], &stack[0][44..51]].map(|fragment| {
        core::fragment_untraced(fragment, &facts).map_err(|error| Refusal::Model {
            event,
            input,
            kind: error.kind(),
        })
    });
    let [birth, continuation] = traces;
    let trace =
        core::leaf_untraced(&stack[0], &facts, &member.key.0).map_err(|error| Refusal::Model {
            event,
            input,
            kind: error.kind(),
        })?;
    let relations = [
        (Relation::BornFirst, birth?),
        (Relation::ContinueFirst, continuation?),
    ]
    .map(|(relation, trace)| RelationEvidence {
        event,
        input,
        member,
        relation,
        leaf: stack[0].clone(),
        control: control.clone(),
        primitive_count: trace.primitive_count,
        peak: trace.peak,
        maximum_width: trace.maximum_width,
    });
    Ok((
        relations,
        LeafEvidence {
            event,
            input,
            member,
            primitive_count: trace.primitive_count,
            peak: trace.peak,
            maximum_width: trace.maximum_width,
        },
    ))
}
