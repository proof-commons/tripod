//! Bounded conformance model evaluation, never native execution or target admission.
//!
//! Source trace: target-elements/src/opcode.rs input introspection (1001,
//! 1061), current index (1162), output script (1257), and `witness_or_digest_program`;
//! encoding.rs:690-698 fixes witness programs at 2–40 bytes; :709-735
//! fixes txids, four-byte indexes and one-byte flags. DROP (opcode.rs:1902),
//! ROT (:1882-1887) and EQUALVERIFY (:1950-1963) follow their operand contracts. Bytes below are decoded
//! directly, not through tapscript's decoder/evaluator/projector. Introspection
//! facts are explicit offline operands; no transaction or scriptPubKey parser,
//! signature implementation, control-path authentication or hashing is claimed.

#[derive(Clone, Debug)]
pub enum ProgramFact {
    Unavailable,
    Witness { version: u8, payload: Vec<u8> },
    Other { sha256: [u8; 32] },
}
impl ProgramFact {
    pub fn rows(&self) -> Result<[Vec<u8>; 2], Refusal> {
        match self {
            Self::Unavailable => Err(Refusal::Fact("missing_program")),
            Self::Witness { version, payload }
                if *version <= 16 && (2..=40).contains(&payload.len()) =>
            {
                Ok([payload.clone(), number(u32::from(*version))])
            }
            Self::Other { sha256 } => Ok([sha256.to_vec(), vec![0x81]]),
            Self::Witness { .. } => Err(Refusal::Fact("witness_program")),
        }
    }
}
#[derive(Clone, Debug)]
pub struct InputFact {
    pub txid: [u8; 32],
    pub index: u32,
    pub flags: u8,
    pub program: ProgramFact,
}
#[derive(Clone, Debug)]
pub struct Facts {
    pub enabled: bool,
    pub current: Option<u32>,
    pub inputs: Vec<InputFact>,
    pub outputs: Vec<ProgramFact>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    Fact(&'static str),
    Domain,
    Unsupported(u8),
    Truncated,
    Underflow,
    Context,
    Index,
    Number,
    Unequal,
    Residue,
    RecipeKey,
    Bound,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trace {
    // Main stack after every instruction, bottom first; alternate is always empty.
    pub main: Vec<Vec<Vec<u8>>>,
    pub primitive_count: u64,
    pub peak: usize,
    pub maximum_width: usize,
}
pub fn number(mut n: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    while n != 0 {
        bytes.push((n & 255) as u8);
        n >>= 8;
    }
    if bytes.last().is_some_and(|b| b & 0x80 != 0) {
        bytes.push(0);
    }
    bytes
}
pub fn index(bytes: &[u8]) -> Result<usize, Refusal> {
    if bytes.len() > 4
        || bytes.last().is_some_and(|b| {
            b.trailing_zeros() >= 7 && (bytes.len() == 1 || bytes[bytes.len() - 2] & 0x80 == 0)
        })
    {
        return Err(Refusal::Number);
    }
    if bytes.last().is_some_and(|b| b & 0x80 != 0) {
        return Err(Refusal::Index);
    }
    let n = bytes
        .iter()
        .enumerate()
        .fold(0_u32, |n, (i, b)| n | (u32::from(*b) << (8 * i)));
    if number(n) != bytes {
        return Err(Refusal::Number);
    }
    usize::try_from(n).map_err(|_| Refusal::Index)
}
impl Refusal {
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Fact(_) => "malformed_offline_fact",
            Self::Domain => "unsupported_execution_domain",
            Self::Unsupported(_) => "unsupported_opcode",
            Self::Truncated => "truncated",
            Self::Underflow => "underflow",
            Self::Context => "context",
            Self::Index => "index",
            Self::Number => "number",
            Self::Unequal => "unequal",
            Self::Residue => "residue",
            Self::RecipeKey => "recipe_key",
            Self::Bound => "bound",
        }
    }
}
fn pop(stack: &mut Vec<Vec<u8>>) -> Result<Vec<u8>, Refusal> {
    stack.pop().ok_or(Refusal::Underflow)
}

#[cfg(test)]
pub fn evaluate(script: &[u8], facts: &Facts) -> Result<Trace, Refusal> {
    let _span = tracing::debug_span!(target: "vectors::keyed_stack_model", "evaluate_keyed_model", script_bytes = script.len()).entered();
    let result = execute(script, facts);
    match &result {
        Ok(trace) => {
            tracing::debug!(target: "vectors::keyed_stack_model", outcome = "completed", model_only = true, primitive_count = trace.primitive_count, "keyed model evaluated");
        }
        Err(error) => {
            tracing::debug!(target: "vectors::keyed_stack_model", outcome = "refused", model_only = true, refusal_kind = error.kind(), "keyed model evaluated");
        }
    }
    result
}
fn execute(script: &[u8], facts: &Facts) -> Result<Trace, Refusal> {
    if script.len() > 128 {
        return Err(Refusal::Bound);
    }
    let mut stack = Vec::new();
    let mut trace = Trace {
        main: vec![vec![]],
        primitive_count: 0,
        peak: 0,
        maximum_width: 0,
    };
    let mut pc = 0;
    while pc < script.len() {
        let op = script[pc];
        pc += 1;
        // The reviewed primitive contracts gate every opcode, not only
        // introspection. Literal pushes have no primitive-domain failure.
        if !facts.enabled && matches!(op, 0x75 | 0x7b | 0x88 | 0xcd | 0xc7 | 0xca | 0xd1) {
            return Err(Refusal::Domain);
        }
        match op {
            0..=75 => {
                let end = pc + usize::from(op);
                let value = script.get(pc..end).ok_or(Refusal::Truncated)?;
                stack.push(value.to_vec());
                pc = end;
            }
            0x51 => stack.push(vec![1]),
            0x75 => {
                pop(&mut stack)?;
                trace.primitive_count += 1;
            }
            0x7b => {
                let n = stack.len();
                if n < 3 {
                    return Err(Refusal::Underflow);
                }
                let value = stack.remove(n - 3);
                stack.push(value);
                trace.primitive_count += 1;
            }
            0x88 => {
                let a = pop(&mut stack)?;
                let b = pop(&mut stack)?;
                if a != b {
                    return Err(Refusal::Unequal);
                }
                trace.primitive_count += 1;
            }
            0xcd | 0xc7 | 0xca | 0xd1 => {
                if facts.current.is_none() {
                    return Err(Refusal::Context);
                }
                if op == 0xcd {
                    let current = facts.current.ok_or(Refusal::Context)?;
                    if current > i32::MAX as u32 {
                        return Err(Refusal::Fact("current_index"));
                    }
                    stack.push(number(current));
                } else {
                    let i = index(&pop(&mut stack)?)?;
                    match op {
                        0xc7 => {
                            let input = facts.inputs.get(i).ok_or(Refusal::Index)?;
                            stack.extend([
                                input.txid.to_vec(),
                                input.index.to_le_bytes().to_vec(),
                                vec![input.flags],
                            ]);
                        }
                        0xca => {
                            stack
                                .extend(facts.inputs.get(i).ok_or(Refusal::Index)?.program.rows()?);
                        }
                        0xd1 => stack.extend(facts.outputs.get(i).ok_or(Refusal::Index)?.rows()?),
                        _ => unreachable!(),
                    }
                }
                trace.primitive_count += 1;
            }
            _ => return Err(Refusal::Unsupported(op)),
        }
        trace.peak = trace.peak.max(stack.len());
        trace.maximum_width = trace
            .maximum_width
            .max(stack.iter().map(Vec::len).max().unwrap_or(0));
        trace.main.push(stack.clone());
    }
    Ok(trace)
}

pub fn fragment_untraced(script: &[u8], facts: &Facts) -> Result<Trace, Refusal> {
    let trace = execute(script, facts)?;
    if trace.main.last() != Some(&vec![]) {
        return Err(Refusal::Residue);
    }
    Ok(trace)
}
pub fn leaf_untraced(script: &[u8], facts: &Facts, key: &[u8; 32]) -> Result<Trace, Refusal> {
    // Recipe equality is a commitment boundary, NOT an execution failure:
    // the key is dropped, so changing it does not change raw stack execution.
    if script.first() != Some(&32) || script.get(1..33) != Some(key.as_slice()) {
        return Err(Refusal::RecipeKey);
    }
    let trace = execute(script, facts)?;
    if trace.main.last() != Some(&vec![vec![1]]) {
        return Err(Refusal::Residue);
    }
    Ok(trace)
}

#[cfg(test)]
pub fn fragment(script: &[u8], facts: &Facts) -> Result<Trace, Refusal> {
    let trace = evaluate(script, facts)?;
    if trace.main.last() != Some(&vec![]) {
        return Err(Refusal::Residue);
    }
    Ok(trace)
}
#[cfg(test)]
pub fn leaf(script: &[u8], facts: &Facts, key: &[u8; 32]) -> Result<Trace, Refusal> {
    if script.first() != Some(&32) || script.get(1..33) != Some(key.as_slice()) {
        return Err(Refusal::RecipeKey);
    }
    let trace = evaluate(script, facts)?;
    if trace.main.last() != Some(&vec![vec![1]]) {
        return Err(Refusal::Residue);
    }
    Ok(trace)
}
