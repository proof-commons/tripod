//! Bounded offline model of the record's byte schedule, independent of emission.
//!
//! Hash contexts retain their transcript here; their reported width is the
//! source-derived serialized width, not a claim to emulate context serialization.

pub use crate::keyed_stack_core::ProgramFact;
use crate::keyed_stack_core::{index, number};
use target_elements_conformance::constructor::tagged::sha256;

/// Introspection payload and form prefix, already authenticated by the caller.
#[derive(Clone, Debug)]
pub struct Field {
    pub payload: Vec<u8>,
    pub prefix: u8,
}

/// Offline spent/created output facts. Nonce is deliberately absent.
#[derive(Clone, Debug)]
pub struct Output {
    pub program: ProgramFact,
    pub asset: Field,
    pub value: Field,
}

/// Authenticated birth information; flags are data, not predicates.
#[derive(Clone, Debug)]
pub struct Input {
    pub txid: [u8; 32],
    pub birth: u32,
    pub flags: u8,
    pub output: Output,
}

/// Synthetic facts, never a chain-evidence or target-admission receipt.
#[derive(Clone, Debug)]
pub struct Facts {
    pub current: u32,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
}

/// Exact model instruction boundary of refusal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    pub instruction: usize,
    pub kind: &'static str,
}

/// Stack values distinguish bytes from the model's symbolic streaming context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Bytes(Vec<u8>),
    Context(Vec<u8>),
}

/// Complete per-instruction stack trace with derived context widths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trace {
    pub stacks: Vec<Vec<Item>>,
    pub primitives: usize,
    pub peak: usize,
    pub width: usize,
}

fn pop(stack: &mut Vec<Item>) -> Result<Vec<u8>, &'static str> {
    match stack.pop() {
        Some(Item::Bytes(bytes)) => Ok(bytes),
        Some(Item::Context(_)) => Err("context-as-bytes"),
        None => Err("underflow"),
    }
}

fn field(stack: &mut Vec<Item>, value: &Field, asset: bool) -> Result<(), &'static str> {
    let width = match (asset, value.prefix) {
        (true, 1 | 10 | 11) | (false, 8 | 9) => 32,
        (false, 1) => 8,
        _ => return Err("field-form"),
    };
    if value.payload.len() != width {
        return Err("field-width");
    }
    stack.extend([
        Item::Bytes(value.payload.clone()),
        Item::Bytes(vec![value.prefix]),
    ]);
    Ok(())
}

fn instruction(op: u8, stack: &mut Vec<Item>, facts: &Facts) -> Result<(), &'static str> {
    match op {
        0x75 => {
            stack.pop().ok_or("underflow")?;
        }
        0x76 => {
            stack.push(stack.last().ok_or("underflow")?.clone());
        }
        0x7b => {
            let position = stack.len().checked_sub(3).ok_or("underflow")?;
            let value = stack.remove(position);
            stack.push(value);
        }
        0x7e => {
            let right = pop(stack)?;
            let mut left = pop(stack)?;
            left.extend(right);
            if left.len() > 520 {
                return Err("element-bound");
            }
            stack.push(Item::Bytes(left));
        }
        0x88 => {
            if pop(stack)? != pop(stack)? {
                return Err("unequal");
            }
        }
        0xc4 => {
            let message = pop(stack)?;
            stack.push(Item::Context(message));
        }
        0xc6 => {
            let tail = pop(stack)?;
            let Some(Item::Context(mut message)) = stack.pop() else {
                return Err("hash-context");
            };
            message.extend(tail);
            stack.push(Item::Bytes(sha256(&message).to_vec()));
        }
        0xcd => stack.push(Item::Bytes(number(facts.current))),
        0xc7..=0xca | 0xce | 0xcf | 0xd1 => {
            let selected = index(&pop(stack)?).map_err(|_| "index-number")?;
            if op == 0xc7 {
                let input = facts.inputs.get(selected).ok_or("input-index")?;
                stack.extend(
                    [
                        input.txid.to_vec(),
                        input.birth.to_le_bytes().to_vec(),
                        vec![input.flags],
                    ]
                    .map(Item::Bytes),
                );
            } else {
                let output = if op <= 0xca {
                    &facts.inputs.get(selected).ok_or("input-index")?.output
                } else {
                    facts.outputs.get(selected).ok_or("output-index")?
                };
                match op {
                    0xc8 | 0xce => field(stack, &output.asset, true)?,
                    0xc9 | 0xcf => field(stack, &output.value, false)?,
                    _ => stack.extend(
                        output
                            .program
                            .rows()
                            .map_err(|_| "program")?
                            .map(Item::Bytes),
                    ),
                }
            }
        }
        _ => return Err("unsupported-opcode"),
    }
    Ok(())
}

/// Evaluate at most 128 bytes with empty arguments and sole final true.
/// # Errors
/// Reports the exact instruction and model refusal, never native execution.
pub fn evaluate(script: &[u8], facts: &Facts) -> Result<Trace, Refusal> {
    let _span = tracing::debug_span!(target: "vectors::keyed_fixed_stack_model", "evaluate", script_bytes = script.len()).entered();
    let result = evaluate_core(script, facts);
    tracing::debug!(target: "vectors::keyed_fixed_stack_model", outcome = if result.is_ok() { "completed" } else { "refused" }, model_only = true, "fixed-form model evaluated");
    result
}

fn evaluate_core(script: &[u8], facts: &Facts) -> Result<Trace, Refusal> {
    if script.len() > 128 {
        return Err(Refusal {
            instruction: 0,
            kind: "script-bound",
        });
    }
    let mut trace = Trace {
        stacks: vec![vec![]],
        primitives: 0,
        peak: 0,
        width: 0,
    };
    let mut stack = Vec::new();
    let mut pc = 0;
    while let Some(&op) = script.get(pc) {
        pc += 1;
        let refusal = |kind| Refusal {
            instruction: trace.stacks.len() - 1,
            kind,
        };
        match op {
            0..=75 => {
                let end = pc + usize::from(op);
                stack.push(Item::Bytes(
                    script
                        .get(pc..end)
                        .ok_or_else(|| refusal("truncated"))?
                        .to_vec(),
                ));
                pc = end;
            }
            0x51 => stack.push(Item::Bytes(vec![1])),
            _ => {
                instruction(op, &mut stack, facts).map_err(refusal)?;
                trace.primitives += 1;
            }
        }
        trace.peak = trace.peak.max(stack.len());
        for item in &stack {
            let width = match item {
                Item::Bytes(bytes) => bytes.len(),
                Item::Context(bytes) => 40 + bytes.len() % 64,
            };
            trace.width = trace.width.max(width);
        }
        trace.stacks.push(stack.clone());
    }
    if stack != [Item::Bytes(vec![1])] {
        return Err(Refusal {
            instruction: trace.stacks.len() - 1,
            kind: "final-stack",
        });
    }
    Ok(trace)
}
