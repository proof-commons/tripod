//! Semantic fragment contracts and independent concrete mutation oracles.

use std::collections::{BTreeMap, BTreeSet};

use realization::{
    Cycle, Maturity, ProtocolAmount, StateMetadata, StateRepresentationNonce, encode_state_metadata,
};
use sha2::{Digest, Sha256};
use target_elements::{FailureCause, OpcodeId};

use super::reviewed_target;
use crate::pattern::{fragment_prerequisites, op};
use crate::state_announcement::*;
use crate::state_constructor::{STATE_GENERATOR_X, STATE_GENERATOR_Y, STATE_NUMS_KEY};
use crate::{
    AbstractLimits, StackItem, TapscriptInstruction, TapscriptProgram, resource_projection,
    validate_program,
};

fn values(minimum: u64, maximum: u64) -> BTreeMap<StateAnnouncementSymbol, StackItem> {
    use StateAnnouncementSymbol as S;
    let target = reviewed_target();
    let item = |bytes: Vec<u8>| StackItem::new(&target, bytes).unwrap();
    BTreeMap::from([
        (S::InternalKey, item(STATE_NUMS_KEY.to_vec())),
        (S::MaturityLeadMin, item(minimum.to_le_bytes().to_vec())),
        (S::MaturityLeadMax, item(maximum.to_le_bytes().to_vec())),
        (S::StateAsset, item(vec![0x11; 32])),
        (S::StateAmount, StackItem::signed_le64(&target, 1)),
    ])
}

fn bindings(minimum: u64, maximum: u64) -> StateAnnouncementBindings {
    StateAnnouncementBindings::new(&reviewed_target(), values(minimum, maximum)).unwrap()
}

fn metadata(cycle: u64, maturity: Maturity, nonce: u32) -> Vec<u8> {
    encode_state_metadata(
        &StateMetadata {
            omega: ProtocolAmount::new(1).unwrap(),
            y_l: ProtocolAmount::new(2).unwrap(),
            y_t: ProtocolAmount::new(3).unwrap(),
            q: ProtocolAmount::new(4).unwrap(),
            cycle: Cycle::new(cycle),
            maturity,
        },
        StateRepresentationNonce::new(nonce),
    )
}

fn record(id: StateAnnouncementId) -> StateAnnouncementPattern {
    let target = reviewed_target();
    let bindings = bindings(2, 4);
    let fragment = state_announcement_fragment(&target, &bindings, id).unwrap();
    build_state_announcement_pattern(&target, &bindings, id, fragment).unwrap()
}

fn contract(id: StateAnnouncementId) {
    let target = reviewed_target();
    let record = record(id);
    let walked = validate_program(
        &target,
        record.fragment(),
        record.precondition(),
        AbstractLimits::for_target(&target),
    )
    .unwrap();
    assert_eq!(record.id(), &id);
    assert_eq!(record.precondition(), &id.precondition());
    assert_eq!(record.success(), walked.success());
    assert_eq!(record.success().len(), 1);
    assert_eq!(record.nonaborting_failure(), &BTreeSet::new());
    assert_eq!(record.aborts(), walked.aborts());
    assert_eq!(
        record.prerequisites(),
        &fragment_prerequisites(record.fragment())
    );
    assert_eq!(
        record.resources(),
        &resource_projection(&target, record.fragment())
    );
    assert!(record.resources().values().any(|&value| value > 0));
    assert!(!record.aborts().is_empty());
}

#[test]
fn metadata_authentication_contract() {
    contract(StateAnnouncementId::MetadataAuthentication);
}

#[test]
fn maturity_predecessor_contract() {
    contract(StateAnnouncementId::MaturityPredecessor);
}

#[test]
fn lead_window_contract() {
    contract(StateAnnouncementId::LeadWindow);
}

#[test]
fn copy_through_contract() {
    contract(StateAnnouncementId::CopyThrough);
}

#[test]
fn successor_reconstruction_contract() {
    contract(StateAnnouncementId::SuccessorReconstruction);
}

// Logical streaming contexts retain their preimages in this test oracle.
// Curve verification consumes one explicitly observed, independently recomputed
// relation; it makes no abstract claim about arbitrary curve inputs.
struct Oracle {
    stack: Vec<Vec<u8>>,
    program: Vec<u8>,
    asset: Vec<u8>,
    amount: Vec<u8>,
    relation: Option<(Vec<u8>, Vec<u8>, Vec<u8>)>,
}

impl Oracle {
    fn new(stack: Vec<Vec<u8>>) -> Self {
        Self {
            stack,
            program: vec![0; 32],
            asset: vec![0x11; 32],
            amount: 1_i64.to_le_bytes().to_vec(),
            relation: None,
        }
    }

    fn pop(&mut self) -> Result<Vec<u8>, &'static str> {
        self.stack.pop().ok_or("underflow")
    }

    fn number(&mut self) -> Result<i64, &'static str> {
        StackItem::new(&reviewed_target(), self.pop()?)
            .unwrap()
            .script_number_value(&reviewed_target())
            .ok_or("script number")
    }

    fn signed(&mut self) -> Result<i64, &'static str> {
        self.pop()?
            .try_into()
            .map(i64::from_le_bytes)
            .map_err(|_| "integer width")
    }

    fn arithmetic(&mut self, opcode: OpcodeId) -> Result<(), &'static str> {
        use OpcodeId as O;
        let right = self.signed()?;
        let left = self.signed()?;
        if opcode == O::Div64 {
            let quotient = left.checked_div(right).ok_or("overflow")?;
            let remainder = left.checked_rem(right).ok_or("overflow")?;
            self.stack.extend([
                remainder.to_le_bytes().to_vec(),
                quotient.to_le_bytes().to_vec(),
                vec![1],
            ]);
        } else if matches!(opcode, O::Add64 | O::Mul64) {
            let result = if opcode == O::Add64 {
                left.checked_add(right)
            } else {
                left.checked_mul(right)
            };
            if let Some(value) = result {
                self.stack.extend([value.to_le_bytes().to_vec(), vec![1]]);
            } else {
                self.stack.extend([
                    left.to_le_bytes().to_vec(),
                    right.to_le_bytes().to_vec(),
                    vec![],
                ]);
            }
        } else {
            let passes = if opcode == O::LessThanOrEqual64 {
                left <= right
            } else {
                left >= right
            };
            self.stack.push(vec![u8::from(passes)]);
        }
        Ok(())
    }

    fn rearrange(&mut self, opcode: OpcodeId) -> Result<(), &'static str> {
        use OpcodeId as O;
        let depth = self.stack.len();
        match opcode {
            O::Duplicate => self
                .stack
                .push(self.stack.last().ok_or("underflow")?.clone()),
            O::DuplicateTwo => self.stack.extend(self.stack[depth - 2..].to_vec()),
            O::CopyOver => self.stack.push(self.stack[depth - 2].clone()),
            O::Swap => self.stack.swap(depth - 1, depth - 2),
            O::Rotate => {
                let item = self.stack.remove(depth - 3);
                self.stack.push(item);
            }
            O::RemoveSecond => {
                self.stack.remove(depth - 2);
            }
            O::Drop => {
                self.pop()?;
            }
            _ => return Err("unsupported rearrangement"),
        }
        Ok(())
    }

    fn bytes(&mut self, opcode: OpcodeId) -> Result<(), &'static str> {
        use OpcodeId as O;
        match opcode {
            O::Substring => {
                let length = usize::try_from(self.number()?).map_err(|_| "slice")?;
                let start = usize::try_from(self.number()?).map_err(|_| "slice")?;
                let item = self.pop()?;
                self.stack
                    .push(item.get(start..start + length).ok_or("slice")?.to_vec());
            }
            O::Concatenate | O::Sha256Update | O::Sha256Finalize => {
                let right = self.pop()?;
                let mut left = self.pop()?;
                left.extend(right);
                self.stack.push(if opcode == O::Sha256Finalize {
                    Sha256::digest(left).to_vec()
                } else {
                    left
                });
            }
            O::BitwiseAnd | O::BitwiseXor => {
                let right = self.pop()?;
                let left = self.pop()?;
                if left.len() != right.len() {
                    return Err("bitwise width");
                }
                self.stack.push(
                    left.iter()
                        .zip(right)
                        .map(|(a, b)| {
                            if opcode == O::BitwiseAnd {
                                a & b
                            } else {
                                a ^ b
                            }
                        })
                        .collect(),
                );
            }
            O::ScriptNumToLe64 => {
                let value = self.number()?;
                self.stack.push(value.to_le_bytes().to_vec());
            }
            O::Le32ToLe64 => {
                let bytes: [u8; 4] = self.pop()?.try_into().map_err(|_| "integer width")?;
                self.stack
                    .push(i64::from(u32::from_le_bytes(bytes)).to_le_bytes().to_vec());
            }
            O::Sha256Initialize => (),
            _ => return Err("unsupported byte primitive"),
        }
        Ok(())
    }

    fn inspect(&mut self, opcode: OpcodeId) -> Result<(), &'static str> {
        use OpcodeId as O;
        assert_eq!(self.number()?, 0);
        let payload = match opcode {
            O::InspectInputScriptPubKey | O::InspectOutputScriptPubKey => self.program.clone(),
            O::InspectOutputAsset => self.asset.clone(),
            O::InspectOutputValue => self.amount.clone(),
            _ => return Err("unsupported inspection"),
        };
        self.stack.extend([payload, vec![1]]);
        Ok(())
    }

    fn execute(&mut self, program: &TapscriptProgram) -> Result<(), &'static str> {
        use OpcodeId as O;
        for instruction in program.instructions() {
            let TapscriptInstruction::Opcode(opcode) = instruction else {
                if let TapscriptInstruction::Push(item) = instruction {
                    self.stack.push(item.bytes().to_vec());
                }
                continue;
            };
            match *opcode {
                O::EqualVerify => {
                    if self.pop()? != self.pop()? {
                        return Err("equality");
                    }
                }
                O::Verify => {
                    let bytes = self.pop()?;
                    if bytes.iter().all(|byte| *byte == 0) {
                        return Err("verification");
                    }
                }
                O::TweakVerify => {
                    let internal = self.pop()?;
                    let tweak = self.pop()?;
                    let compressed = self.pop()?;
                    if self.relation.as_ref() != Some(&(compressed, tweak, internal)) {
                        return Err("curve relation");
                    }
                }
                O::InspectInputScriptPubKey
                | O::InspectOutputScriptPubKey
                | O::InspectOutputAsset
                | O::InspectOutputValue => self.inspect(*opcode)?,
                O::Add64 | O::Mul64 | O::Div64 | O::LessThanOrEqual64 | O::GreaterThanOrEqual64 => {
                    self.arithmetic(*opcode)?;
                }
                O::Duplicate
                | O::DuplicateTwo
                | O::CopyOver
                | O::Swap
                | O::Rotate
                | O::RemoveSecond
                | O::Drop => self.rearrange(*opcode)?,
                _ => self.bytes(*opcode)?,
            }
        }
        Ok(())
    }
}

// Independent public-point arithmetic, used only to derive this test's golden.
// Four little-endian limbs avoid adding an out-of-scope dependency.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Field([u64; 4]);

const MODULUS: Field = Field([0xffff_fffe_ffff_fc2f, u64::MAX, u64::MAX, u64::MAX]);
const ZERO: Field = Field([0; 4]);
const ONE: Field = Field([1, 0, 0, 0]);

impl Field {
    fn from_bytes(bytes: [u8; 32]) -> Self {
        let mut limbs = [0; 4];
        for (slot, chunk) in limbs.iter_mut().rev().zip(bytes.as_chunks::<8>().0.iter()) {
            *slot = u64::from_be_bytes(*chunk);
        }
        Self(limbs)
    }

    fn bytes(self) -> [u8; 32] {
        let mut bytes = [0; 32];
        for (slot, value) in bytes
            .as_chunks_mut::<8>()
            .0
            .iter_mut()
            .zip(self.0.iter().rev())
        {
            slot.copy_from_slice(&value.to_be_bytes());
        }
        bytes
    }

    fn subtract_raw(self, rhs: Self) -> Self {
        let mut result = [0; 4];
        let mut borrow = false;
        for ((slot, left), right) in result.iter_mut().zip(self.0).zip(rhs.0) {
            let (difference, first) = left.overflowing_sub(right);
            let (difference, second) = difference.overflowing_sub(u64::from(borrow));
            *slot = difference;
            borrow = first || second;
        }
        Self(result)
    }

    fn add(self, rhs: Self) -> Self {
        let mut result = [0; 4];
        let mut carry = false;
        for ((slot, left), right) in result.iter_mut().zip(self.0).zip(rhs.0) {
            let (sum, first) = left.overflowing_add(right);
            let (sum, second) = sum.overflowing_add(u64::from(carry));
            *slot = sum;
            carry = first || second;
        }
        let sum = Self(result);
        if carry || sum.0.iter().rev().cmp(MODULUS.0.iter().rev()).is_ge() {
            sum.subtract_raw(MODULUS)
        } else {
            sum
        }
    }

    fn sub(self, rhs: Self) -> Self {
        self.add(MODULUS.subtract_raw(rhs))
    }

    fn mul(self, rhs: Self) -> Self {
        let mut accumulator = ZERO;
        let mut doubled = self;
        for limb in rhs.0 {
            for bit in 0..64 {
                if limb & (1 << bit) != 0 {
                    accumulator = accumulator.add(doubled);
                }
                doubled = doubled.add(doubled);
            }
        }
        accumulator
    }

    fn pow(self, exponent: [u8; 32]) -> Self {
        let mut result = ONE;
        for byte in exponent {
            for bit in (0..8).rev() {
                result = result.mul(result);
                if byte & (1 << bit) != 0 {
                    result = result.mul(self);
                }
            }
        }
        result
    }

    fn inverse(self) -> Self {
        self.pow(MODULUS.subtract_raw(Self([2, 0, 0, 0])).bytes())
    }
}

#[derive(Clone, Copy)]
struct Point {
    x: Field,
    y: Field,
    z: Field,
}

impl Point {
    fn double(self) -> Self {
        let xx = self.x.mul(self.x);
        let yy = self.y.mul(self.y);
        let yyyy = yy.mul(yy);
        let slope = xx.add(xx).add(xx);
        let twice = self.x.add(yy);
        let spread = twice.mul(twice).sub(xx).sub(yyyy);
        let spread = spread.add(spread);
        let x = slope.mul(slope).sub(spread.add(spread));
        let eight = yyyy.add(yyyy).add(yyyy.add(yyyy));
        let y = slope.mul(spread.sub(x)).sub(eight.add(eight));
        let z = self.y.mul(self.z);
        Self { x, y, z: z.add(z) }
    }

    fn add_affine(self, x: Field, y: Field) -> Self {
        if self.z == ZERO {
            return Self { x, y, z: ONE };
        }
        let zz = self.z.mul(self.z);
        let horizontal = x.mul(zz).sub(self.x);
        let vertical = y.mul(self.z).mul(zz).sub(self.y);
        if horizontal == ZERO {
            return if vertical == ZERO {
                self.double()
            } else {
                Self {
                    x: ZERO,
                    y: ONE,
                    z: ZERO,
                }
            };
        }
        let hh = horizontal.mul(horizontal);
        let hhh = horizontal.mul(hh);
        let base = self.x.mul(hh);
        let next_x = vertical.mul(vertical).sub(hhh).sub(base.add(base));
        let next_y = vertical.mul(base.sub(next_x)).sub(self.y.mul(hhh));
        Self {
            x: next_x,
            y: next_y,
            z: self.z.mul(horizontal),
        }
    }

    fn compressed(self) -> Vec<u8> {
        assert_ne!(self.z, ZERO);
        let inverse = self.z.inverse();
        let x = self.x.mul(inverse.mul(inverse));
        let y = self.y.mul(inverse.mul(inverse).mul(inverse));
        let mut bytes = vec![2 + u8::from(y.0[0] & 1 != 0)];
        bytes.extend(x.bytes());
        bytes
    }
}

pub(super) fn output_key(tweak: [u8; 32]) -> Vec<u8> {
    let generator_x = Field::from_bytes(STATE_GENERATOR_X);
    let generator_y = Field::from_bytes(STATE_GENERATOR_Y);
    let mut point = Point {
        x: ZERO,
        y: ONE,
        z: ZERO,
    };
    for byte in tweak {
        for bit in (0..8).rev() {
            point = point.double();
            if byte & (1 << bit) != 0 {
                point = point.add_affine(generator_x, generator_y);
            }
        }
    }
    let internal_x = Field::from_bytes(STATE_NUMS_KEY);
    let square = internal_x
        .mul(internal_x)
        .mul(internal_x)
        .add(Field([7, 0, 0, 0]));
    let mut exponent = MODULUS.bytes();
    // (p + 1) / 4, shifted explicitly in big-endian representation.
    exponent[31] += 1;
    let mut carry = 0;
    for byte in &mut exponent {
        let low = *byte & 3;
        *byte = (*byte >> 2) | carry;
        carry = low << 6;
    }
    let root = square.pow(exponent);
    assert_eq!(root.mul(root), square);
    let internal_y = if root.0[0] & 1 == 0 {
        root
    } else {
        ZERO.sub(root)
    };
    point.add_affine(internal_x, internal_y).compressed()
}

pub(super) fn tagged(tag: &[u8], payload: &[u8]) -> [u8; 32] {
    let tag = Sha256::digest(tag);
    let mut hash = Sha256::new();
    hash.update(tag);
    hash.update(tag);
    hash.update(payload);
    hash.finalize().into()
}

fn golden(bytes: &[u8], static_root: [u8; 32]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut script = vec![0x4c, 86];
    script.extend(bytes);
    script.extend([0, 0x69]);
    assert_eq!(script.len(), 90);
    let mut leaf_preimage = vec![0xc4, 90];
    leaf_preimage.extend(script);
    let leaf = tagged(b"TapLeaf/elements", &leaf_preimage);
    assert!(leaf < static_root);
    let root = tagged(b"TapBranch/elements", &[leaf, static_root].concat());
    let tweak = tagged(b"TapTweak/elements", &[STATE_NUMS_KEY, root].concat());
    (output_key(tweak), tweak.to_vec(), STATE_NUMS_KEY.to_vec())
}

fn authenticated_oracle(bytes: Vec<u8>) -> Oracle {
    authenticated_oracle_with_root(bytes, [0xff; 32])
}

fn authenticated_oracle_with_root(bytes: Vec<u8>, static_root: [u8; 32]) -> Oracle {
    let relation = golden(&bytes, static_root);
    let mut oracle = Oracle::new(vec![static_root.to_vec(), bytes, vec![relation.0[0]]]);
    oracle.program = relation.0[1..].to_vec();
    oracle.relation = Some(relation);
    oracle
}

#[test]
fn golden_recomputes_leaf_root_tweak_and_output_key() {
    let bytes = metadata(5, Maturity::Unannounced, 1);
    let program = record(StateAnnouncementId::MetadataAuthentication);
    let mut oracle = authenticated_oracle(bytes.clone());
    assert_eq!(oracle.execute(program.fragment()), Ok(()));
    assert_eq!(oracle.stack, vec![vec![0xff; 32], bytes]);
    let relation = oracle.relation.unwrap();
    for position in 0..32 {
        let mut wrong = Oracle::new(vec![
            vec![0xff; 32],
            metadata(5, Maturity::Unannounced, 1),
            vec![relation.0[0]],
        ]);
        wrong.program = relation.0[1..].to_vec();
        wrong.program[position] ^= 1;
        wrong.relation = Some(relation.clone());
        assert_eq!(wrong.execute(program.fragment()), Err("curve relation"));
    }
}

#[test]
fn authentication_rejects_every_metadata_byte_and_wrong_parity() {
    let bytes = metadata(5, Maturity::Unannounced, 1);
    let base = authenticated_oracle(bytes.clone());
    let program = record(StateAnnouncementId::MetadataAuthentication);
    for position in 0..bytes.len() {
        let mut wrong = Oracle::new(base.stack.clone());
        wrong.stack[1][position] ^= 1;
        wrong.program.clone_from(&base.program);
        wrong.relation.clone_from(&base.relation);
        assert_eq!(wrong.execute(program.fragment()), Err("curve relation"));
    }
    let mut wrong = base;
    wrong.stack[2][0] ^= 1;
    assert_eq!(wrong.execute(program.fragment()), Err("curve relation"));
}

#[test]
fn wrong_witnessed_root_and_internal_key_do_not_authenticate() {
    let base = authenticated_oracle(metadata(5, Maturity::Unannounced, 1));
    for id in [
        StateAnnouncementId::MetadataAuthentication,
        StateAnnouncementId::SuccessorReconstruction,
    ] {
        let canonical = record(id);
        for position in 0..32 {
            let mut wrong = Oracle::new(base.stack.clone());
            wrong.stack[0][position] ^= 1;
            wrong.program.clone_from(&base.program);
            wrong.relation.clone_from(&base.relation);
            assert_eq!(wrong.execute(canonical.fragment()), Err("curve relation"));
        }
        let mut substitutions = values(2, 4);
        substitutions.insert(
            StateAnnouncementSymbol::InternalKey,
            StackItem::new(&reviewed_target(), vec![0x33; 32]).unwrap(),
        );
        let bindings = StateAnnouncementBindings::new(&reviewed_target(), substitutions).unwrap();
        let program = state_announcement_fragment(&reviewed_target(), &bindings, id).unwrap();
        let mut wrong = Oracle::new(base.stack.clone());
        wrong.program.clone_from(&base.program);
        wrong.relation.clone_from(&base.relation);
        assert_eq!(wrong.execute(&program), Err("curve relation"));
    }
}

#[test]
fn only_unannounced_discriminant_survives() {
    let record = record(StateAnnouncementId::MaturityPredecessor);
    for maturity in [
        Maturity::Unannounced,
        Maturity::Announced {
            cycle: Cycle::new(7),
        },
        Maturity::Complete,
    ] {
        let bytes = metadata(5, maturity, 0);
        let mut oracle = Oracle::new(vec![bytes.clone()]);
        assert_eq!(
            oracle.execute(record.fragment()).is_ok(),
            maturity == Maturity::Unannounced
        );
        if maturity == Maturity::Unannounced {
            assert_eq!(oracle.stack, vec![bytes]);
        }
    }
}

fn check_window(current: u64, requested: u64, minimum: u64, maximum: u64) {
    let target = reviewed_target();
    let fragment = state_announcement_fragment(
        &target,
        &bindings(minimum, maximum),
        StateAnnouncementId::LeadWindow,
    )
    .unwrap();
    let witness = vec![
        metadata(current, Maturity::Unannounced, 0),
        requested.to_be_bytes().to_vec(),
    ];
    let mut oracle = Oracle::new(witness.clone());
    let expected = current
        .checked_add(minimum)
        .zip(current.checked_add(maximum))
        .is_some_and(|(lower, upper)| lower <= requested && requested <= upper);
    assert_eq!(
        oracle.execute(&fragment).is_ok(),
        expected,
        "current={current}, requested={requested}, bounds={minimum}..={maximum}"
    );
    if expected {
        assert_eq!(oracle.stack, witness);
    }
}

#[test]
fn lead_window_is_inclusive_and_rejects_both_overflows() {
    for requested in 5..=10 {
        check_window(5, requested, 2, 4);
    }
    for current in [u64::MAX - 4, u64::MAX - 3, u64::MAX - 1, u64::MAX] {
        check_window(current, u64::MAX, 2, 4);
    }
}

#[test]
fn byte_order_bridge_covers_signed_boundary_and_full_unsigned_leads() {
    for current in [
        0,
        255,
        65_535,
        0x0102_0304_0506_0708,
        (1 << 63) - 1,
        1 << 63,
        u64::MAX - 4,
    ] {
        for delta in [1, 2, 3, 4, 5] {
            if let Some(requested) = current.checked_add(delta) {
                check_window(current, requested, 2, 4);
            }
        }
    }
    for lead in [(1 << 63) - 1, 1 << 63, (1 << 63) + 1, u64::MAX] {
        for current in [0, 1, 3] {
            check_window(current, lead, lead, lead);
            check_window(current, u64::MAX, lead, lead);
        }
    }
}

#[test]
fn copy_through_matches_codec_without_an_equality_census() {
    let fragment = record(StateAnnouncementId::CopyThrough);
    assert!(
        !fragment
            .fragment()
            .instructions()
            .contains(&op(OpcodeId::EqualVerify))
    );
    for (cycle, requested, nonce) in [
        (5, 7, 1),
        (255, 65_536, 0x0102_0304),
        (u64::MAX - 1, u64::MAX, u32::MAX),
    ] {
        let previous = metadata(cycle, Maturity::Unannounced, 23);
        let expected = metadata(
            cycle,
            Maturity::Announced {
                cycle: Cycle::new(requested),
            },
            nonce,
        );
        let mut oracle = Oracle::new(vec![
            nonce.to_be_bytes().to_vec(),
            vec![0xff; 32],
            requested.to_be_bytes().to_vec(),
            previous,
        ]);
        assert_eq!(oracle.execute(fragment.fragment()), Ok(()));
        assert_eq!(oracle.stack, vec![vec![0xff; 32], expected]);
    }
}

#[test]
fn successor_authenticates_exact_asset_amount_program_and_retains_metadata() {
    let bytes = metadata(
        5,
        Maturity::Announced {
            cycle: Cycle::new(7),
        },
        1,
    );
    let fragment = record(StateAnnouncementId::SuccessorReconstruction);
    let base = authenticated_oracle(bytes.clone());
    for field in 0..4 {
        let mut oracle = Oracle::new(base.stack.clone());
        oracle.program.clone_from(&base.program);
        oracle.relation.clone_from(&base.relation);
        match field {
            0 => (),
            1 => oracle.asset[0] ^= 1,
            2 => oracle.amount[0] ^= 1,
            _ => oracle.program[0] ^= 1,
        }
        assert_eq!(oracle.execute(fragment.fragment()).is_ok(), field == 0);
        if field == 0 {
            assert_eq!(oracle.stack, vec![bytes.clone()]);
        }
    }
}

#[test]
fn changed_nonmaturity_bytes_and_wrong_nonce_cannot_reconstruct_output() {
    let bytes = metadata(
        5,
        Maturity::Announced {
            cycle: Cycle::new(7),
        },
        1,
    );
    let base = authenticated_oracle(bytes);
    let fragment = record(StateAnnouncementId::SuccessorReconstruction);
    for position in (0..65).chain(74..86) {
        let mut oracle = Oracle::new(base.stack.clone());
        oracle.stack[1][position] ^= 1;
        oracle.program.clone_from(&base.program);
        oracle.relation.clone_from(&base.relation);
        assert_eq!(oracle.execute(fragment.fragment()), Err("curve relation"));
    }
}

#[test]
fn copy_through_nonce_mutation_is_bound_by_successor_authentication() {
    let successor = metadata(
        5,
        Maturity::Announced {
            cycle: Cycle::new(7),
        },
        1,
    );
    let previous = authenticated_oracle(metadata(5, Maturity::Unannounced, 0));
    let next = authenticated_oracle(successor.clone());
    for nonce in [0_u32, 1, 2, u32::MAX] {
        // The operator has consumed its signature; preserve the remaining witness order.
        let mut oracle = Oracle::new(vec![
            next.stack[2].clone(),
            nonce.to_be_bytes().to_vec(),
            7_u64.to_be_bytes().to_vec(),
            previous.stack[0].clone(),
            previous.stack[1].clone(),
            previous.stack[2].clone(),
        ]);
        oracle.program.clone_from(&previous.program);
        oracle.relation.clone_from(&previous.relation);
        oracle
            .execute(record(StateAnnouncementId::MetadataAuthentication).fragment())
            .unwrap();
        oracle
            .execute(record(StateAnnouncementId::MaturityPredecessor).fragment())
            .unwrap();
        oracle.rearrange(OpcodeId::Rotate).unwrap();
        oracle
            .execute(record(StateAnnouncementId::LeadWindow).fragment())
            .unwrap();
        oracle.rearrange(OpcodeId::Swap).unwrap();
        oracle
            .execute(record(StateAnnouncementId::CopyThrough).fragment())
            .unwrap();
        assert_eq!(oracle.stack[0], next.stack[2]);
        assert_eq!(oracle.stack[1], previous.stack[0]);
        assert_eq!(oracle.stack.len(), 3);
        oracle.rearrange(OpcodeId::Rotate).unwrap();
        oracle.program.clone_from(&next.program);
        oracle.relation.clone_from(&next.relation);
        assert_eq!(
            oracle
                .execute(record(StateAnnouncementId::SuccessorReconstruction).fragment())
                .is_ok(),
            nonce == 1
        );
        if nonce == 1 {
            assert_eq!(oracle.stack, vec![successor.clone()]);
        }
    }
}

#[test]
fn authentication_retains_exact_witnessed_root_beneath_metadata() {
    let bytes = metadata(5, Maturity::Unannounced, 1);
    let mut root = [0xff; 32];
    root[1] = 0x31;
    root[31] = 0x75;
    let mut oracle = authenticated_oracle_with_root(bytes.clone(), root);
    let pattern = record(StateAnnouncementId::MetadataAuthentication);
    assert!(
        pattern
            .metadata()
            .disclosure
            .contains(&StateAnnouncementWitness::StaticSubtreeRoot)
    );
    assert_eq!(oracle.execute(pattern.fragment()), Ok(()));
    assert_eq!(oracle.stack, vec![root.to_vec(), bytes]);
}

#[test]
fn successor_consumes_witnessed_root_without_leaving_a_copy() {
    let bytes = metadata(
        5,
        Maturity::Announced {
            cycle: Cycle::new(7),
        },
        1,
    );
    let mut oracle = authenticated_oracle(bytes.clone());
    let pattern = record(StateAnnouncementId::SuccessorReconstruction);
    assert!(
        pattern
            .metadata()
            .disclosure
            .contains(&StateAnnouncementWitness::StaticSubtreeRoot)
    );
    assert_eq!(oracle.execute(pattern.fragment()), Ok(()));
    assert_eq!(oracle.stack, vec![bytes]);
}

#[test]
fn recipe_metadata_and_consumers_are_exact_component_unions() {
    let recipe = state_announcement_patterns(&reviewed_target(), &bindings(2, 4)).unwrap();
    assert_eq!(recipe.components().len(), 5);
    let mut union = StateAnnouncementMetadata::default();
    for component in recipe.components() {
        union.sources.extend(&component.metadata().sources);
        union.evidence.extend(&component.metadata().evidence);
        union.disclosure.extend(&component.metadata().disclosure);
        union.residuals.extend(&component.metadata().residuals);
        for (&symbol, requirement) in component.consumers() {
            assert_eq!(symbol, requirement.symbol);
            for (id, sites) in &requirement.sites {
                assert_eq!(id, component.id());
                assert!(!sites.is_empty());
                for &site in sites {
                    assert_eq!(
                        &component.fragment().instructions()[site],
                        &TapscriptInstruction::Push(values(2, 4)[&symbol].clone())
                    );
                }
                assert_eq!(recipe.consumers()[&symbol].sites[id], *sites);
            }
        }
    }
    assert_eq!(recipe.metadata(), &union);
    assert_eq!(
        recipe.consumers().keys().copied().collect::<BTreeSet<_>>(),
        StateAnnouncementSymbol::ALL.iter().copied().collect()
    );
    assert_eq!(StateAnnouncementOwner::ALL.len(), 5);
    assert_eq!(StateAnnouncementWitness::ALL.len(), 6);
    assert_eq!(StateAnnouncementSymbol::ALL.len(), 5);
}

#[test]
fn binding_census_rejects_missing_width_domain_and_bound_order() {
    use StateAnnouncementSymbol as S;
    let target = reviewed_target();
    for &symbol in S::ALL {
        let mut missing = values(2, 4);
        missing.remove(&symbol);
        assert_eq!(
            StateAnnouncementBindings::new(&target, missing),
            Err(StateAnnouncementRefusal::ConsumerCensus)
        );
        let mut invalid = values(2, 4);
        invalid.insert(symbol, StackItem::new(&target, vec![1; 7]).unwrap());
        assert_eq!(
            StateAnnouncementBindings::new(&target, invalid),
            Err(StateAnnouncementRefusal::InvalidBinding(symbol))
        );
    }
    for (minimum, maximum) in [(0, 1), (5, 4)] {
        assert_eq!(
            StateAnnouncementBindings::new(&target, values(minimum, maximum)),
            Err(StateAnnouncementRefusal::InvalidBinding(S::MaturityLeadMin))
        );
    }
    for symbol in [S::StateAsset, S::StateAmount] {
        assert!(symbol.structural().is_some());
    }
}

#[test]
fn mutated_fragments_and_incompatible_recipes_cannot_inherit_identity() {
    let target = reviewed_target();
    for &id in StateAnnouncementId::ALL {
        let record = record(id);
        let mut instructions = record.fragment().instructions().to_vec();
        instructions.push(op(OpcodeId::Duplicate));
        assert_eq!(
            build_state_announcement_pattern(
                &target,
                &bindings(2, 4),
                id,
                TapscriptProgram::new(instructions).unwrap()
            ),
            Err(StateAnnouncementRefusal::FragmentMismatch)
        );
    }
    let recipe = state_announcement_patterns(&target, &bindings(2, 4)).unwrap();
    let mut incomplete = recipe.components().clone();
    incomplete.pop();
    assert_eq!(
        StateAnnouncementRecipe::new(incomplete),
        Err(StateAnnouncementRefusal::ComponentRecipe)
    );
    let mut reordered = recipe.components().clone();
    reordered.swap(0, 1);
    assert_eq!(
        StateAnnouncementRecipe::new(reordered),
        Err(StateAnnouncementRefusal::ComponentRecipe)
    );
    let alternate = state_announcement_patterns(&target, &bindings(3, 4)).unwrap();
    let mut incompatible = recipe.components().clone();
    incompatible[0] = alternate.components()[0].clone();
    assert_eq!(
        StateAnnouncementRecipe::new(incompatible),
        Err(StateAnnouncementRefusal::ComponentRecipe)
    );
}

#[test]
fn every_boolean_is_immediately_verified_and_aborts_are_named() {
    let recipe = state_announcement_patterns(&reviewed_target(), &bindings(2, 4)).unwrap();
    for record in recipe.components() {
        for pair in record.fragment().instructions().windows(2) {
            if matches!(
                &pair[0],
                TapscriptInstruction::Opcode(
                    OpcodeId::Add64
                        | OpcodeId::Mul64
                        | OpcodeId::Div64
                        | OpcodeId::LessThanOrEqual64
                        | OpcodeId::GreaterThanOrEqual64
                )
            ) {
                assert_eq!(pair[1], op(OpcodeId::Verify));
            }
        }
        assert!(
            !record
                .fragment()
                .instructions()
                .contains(&op(OpcodeId::Equal))
        );
    }
    for id in [
        StateAnnouncementId::MetadataAuthentication,
        StateAnnouncementId::SuccessorReconstruction,
    ] {
        assert!(
            record(id)
                .aborts()
                .contains(&FailureCause::InvalidCurveRelation)
        );
    }
    assert!(
        record(StateAnnouncementId::MaturityPredecessor)
            .aborts()
            .contains(&FailureCause::UnequalOperands)
    );
    assert!(
        record(StateAnnouncementId::LeadWindow)
            .aborts()
            .contains(&FailureCause::FalseVerification)
    );
    assert!(
        record(StateAnnouncementId::CopyThrough)
            .aborts()
            .contains(&FailureCause::SliceOutOfRange)
    );
}

#[test]
fn every_semantic_fragment_refuses_missing_items_and_exposes_extra_item_residue() {
    use super::state_program_tests::assert_witness_changes_success;
    use target_elements::StackValueType;
    for &id in StateAnnouncementId::ALL {
        let record = record(id);
        for index in 0..record.precondition().main().len() {
            let mut witness = record.precondition().main().to_vec();
            witness.remove(index);
            assert_witness_changes_success(record.fragment(), witness, record.success());
        }
        for index in 0..=record.precondition().main().len() {
            let mut witness = record.precondition().main().to_vec();
            witness.insert(
                index,
                StackValueType::Bytes {
                    minimum: 0,
                    maximum: 0,
                },
            );
            assert_witness_changes_success(record.fragment(), witness, record.success());
        }
    }
}

#[test]
fn retained_metadata_and_window_witness_width_mutations_change_declared_success() {
    use super::state_program_tests::assert_witness_changes_success;
    use target_elements::StackValueType;
    for id in [
        StateAnnouncementId::MaturityPredecessor,
        StateAnnouncementId::LeadWindow,
    ] {
        let record = record(id);
        for (index, item) in record.precondition().main().iter().enumerate() {
            let StackValueType::Bytes { minimum, maximum } = item else {
                panic!("fixed byte witness")
            };
            assert_eq!(minimum, maximum);
            for width in [minimum - 1, maximum + 1] {
                let mut witness = record.precondition().main().to_vec();
                witness[index] = StackValueType::Bytes {
                    minimum: width,
                    maximum: width,
                };
                assert_witness_changes_success(record.fragment(), witness, record.success());
            }
        }
        for index in 1..record.precondition().main().len() {
            let mut witness = record.precondition().main().to_vec();
            witness.swap(index - 1, index);
            assert_witness_changes_success(record.fragment(), witness, record.success());
        }
    }
}

#[test]
fn omitted_final_window_verify_leaves_a_boolean_and_cannot_inherit_identity() {
    let target = reviewed_target();
    let record = record(StateAnnouncementId::LeadWindow);
    let mut instructions = record.fragment().instructions().to_vec();
    assert_eq!(instructions.pop(), Some(op(OpcodeId::Verify)));
    let changed = TapscriptProgram::new(instructions).unwrap();
    let walk = validate_program(
        &target,
        &changed,
        record.precondition(),
        AbstractLimits::for_target(&target),
    )
    .unwrap();
    let mut expected = record.precondition().main().to_vec();
    expected.push(target_elements::StackValueType::Bool);
    assert_eq!(
        walk.success(),
        &BTreeSet::from([crate::AbstractStackState::from_main(expected)])
    );
    assert_eq!(
        build_state_announcement_pattern(
            &target,
            &bindings(2, 4),
            StateAnnouncementId::LeadWindow,
            changed
        ),
        Err(StateAnnouncementRefusal::FragmentMismatch)
    );
}

#[test]
fn semantic_refusal_declaration_has_exact_exercised_and_unreachable_census() {
    super::state_program_tests::assert_refusal_census(
        include_str!("../state_announcement.rs"),
        "StateAnnouncementRefusal",
        &[
            (
                "ConsumerCensus",
                binding_census_rejects_missing_width_domain_and_bound_order,
            ),
            (
                "InvalidBinding",
                binding_census_rejects_missing_width_domain_and_bound_order,
            ),
            (
                "FragmentMismatch",
                mutated_fragments_and_incompatible_recipes_cannot_inherit_identity,
            ),
            (
                "ComponentRecipe",
                mutated_fragments_and_incompatible_recipes_cannot_inherit_identity,
            ),
        ],
        &[
            (
                "Program",
                "Fixed checked bindings and canonical emission expose no failing walk; needs an internal walk seam.",
            ),
            (
                "InvalidContract",
                "Exact recipe matching rejects corruption first; needs an internal contract checker.",
            ),
        ],
    );
}

#[test]
fn lead_cross_product_checks_both_endpoints_and_unsigned_overflow() {
    let cycles = [
        0,
        1,
        1 << 32,
        (1 << 63) - 1,
        1 << 63,
        u64::MAX - 1,
        u64::MAX,
    ];
    let leads = [1, (1 << 63) - 1, 1 << 63, u64::MAX];
    for current in cycles {
        for minimum in leads {
            for maximum in leads.into_iter().filter(|maximum| *maximum >= minimum) {
                let lower = current.checked_add(minimum);
                let upper = current.checked_add(maximum);
                for requested in [lower.unwrap_or(u64::MAX), upper.unwrap_or(u64::MAX)] {
                    check_window(current, requested, minimum, maximum);
                }
                if let Some(below) = lower.and_then(|value| value.checked_sub(1)) {
                    check_window(current, below, minimum, maximum);
                }
                if let Some(above) = upper.and_then(|value| value.checked_add(1)) {
                    check_window(current, above, minimum, maximum);
                }
            }
        }
    }
}

#[test]
fn every_reversed_extreme_lead_pair_is_a_named_binding_refusal() {
    let target = reviewed_target();
    for minimum in [1, (1 << 63) - 1, 1 << 63, u64::MAX] {
        for maximum in [0, 1, (1 << 63) - 1, 1 << 63, u64::MAX] {
            if minimum > maximum {
                assert_eq!(
                    StateAnnouncementBindings::new(&target, values(minimum, maximum)),
                    Err(StateAnnouncementRefusal::InvalidBinding(
                        StateAnnouncementSymbol::MaturityLeadMin
                    ))
                );
            }
        }
    }
}

fn copied_successor(previous: Vec<u8>, requested: u64, nonce: u32) -> Vec<u8> {
    let mut oracle = Oracle::new(vec![
        nonce.to_be_bytes().to_vec(),
        vec![0xff; 32],
        requested.to_be_bytes().to_vec(),
        previous,
    ]);
    oracle
        .execute(record(StateAnnouncementId::CopyThrough).fragment())
        .unwrap();
    assert_eq!(oracle.stack.len(), 2);
    assert_eq!(oracle.stack[0], vec![0xff; 32]);
    oracle.stack.pop().unwrap()
}

fn assert_successor_refused(bytes: Vec<u8>, base: &Oracle) {
    let mut oracle = Oracle::new(vec![base.stack[0].clone(), bytes, base.stack[2].clone()]);
    oracle.program.clone_from(&base.program);
    oracle.relation.clone_from(&base.relation);
    assert_eq!(
        oracle.execute(record(StateAnnouncementId::SuccessorReconstruction).fragment()),
        Err("curve relation")
    );
}

#[test]
fn every_copied_predecessor_byte_changes_successor_and_breaks_reconstruction() {
    let previous = metadata(5, Maturity::Unannounced, 23);
    let expected = metadata(
        5,
        Maturity::Announced {
            cycle: Cycle::new(7),
        },
        1,
    );
    let base = authenticated_oracle(expected.clone());
    assert_eq!(copied_successor(previous.clone(), 7, 1), expected);
    // The codec's first differing byte locates the maturity discriminant independently.
    let maturity_start = previous
        .iter()
        .zip(&expected)
        .position(|(a, b)| a != b)
        .unwrap();
    assert_eq!(expected[maturity_start], 1);
    for position in 0..maturity_start {
        let mut changed = previous.clone();
        changed[position] ^= 1;
        let successor = copied_successor(changed, 7, 1);
        assert_ne!(successor, expected, "predecessor byte {position}");
        assert_eq!(successor[maturity_start], 1);
        assert_successor_refused(successor, &base);
    }
}

#[test]
fn every_announced_maturity_byte_and_requested_cycle_is_bound_by_reconstruction() {
    let unannounced = metadata(5, Maturity::Unannounced, 1);
    let expected = metadata(
        5,
        Maturity::Announced {
            cycle: Cycle::new(7),
        },
        1,
    );
    let start = unannounced
        .iter()
        .zip(&expected)
        .position(|(a, b)| a != b)
        .unwrap();
    let base = authenticated_oracle(expected.clone());
    for position in start..start + 9 {
        let mut changed = expected.clone();
        changed[position] ^= 1;
        assert_successor_refused(changed, &base);
    }
    for requested in [0, 6, 8, 1 << 63, u64::MAX] {
        let changed = copied_successor(unannounced.clone(), requested, 1);
        assert_ne!(changed, expected);
        assert_successor_refused(changed, &base);
    }
}
