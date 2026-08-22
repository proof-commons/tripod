//! The reviewed rules for pushing a literal onto the target's stack.
//!
//! # Why the push forms are a contract rather than a serializer detail
//!
//! A consumer that emits target programs has to answer three questions
//! about every literal it pushes: which opcode carries it, how the
//! payload's length is written down, and whether that choice is the one
//! the target accepts. All three are target facts, and a backend that
//! answered them from its own constants would be restating the target
//! from memory. So the forms, their opcode spans, the width each form
//! can express, and the rule selecting among them are declared here,
//! and a serializer resolves them rather than reciting them.
//!
//! # Minimality is a relay rule, not a consensus rule
//!
//! The reviewed target enforces the maximum literal size and the
//! well-formedness of a push while validating a spend. It enforces
//! *minimal* push encoding only under the standardness flag a node
//! applies to the transactions it relays. The two are recorded
//! separately below, because a program that violates the first is
//! invalid and a program that violates the second is merely
//! unrelayable, and a consumer that could not tell them apart would
//! either refuse valid programs or emit programs no node forwards.
//!
//! The first-party serializer emits only minimal forms, which is
//! stricter than consensus on purpose.
//!
//! # This module still encodes nothing
//!
//! [`PushContract::minimal_form`] classifies a payload against the
//! reviewed rule and names a form. It produces no bytes, reads no
//! stream, and builds no instruction: applying the contract is not the
//! same as serializing with it, and the serializer lives in the
//! consumer that owns programs.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use crate::capability::census_enum;
use crate::encoding::ByteOrder;
use crate::evidence::TargetEvidenceRequirementId;
use crate::opcode::MAX_STACK_ELEMENT_BYTES;

census_enum! {
    /// One way the target carries a literal payload in a script.
    #[non_exhaustive]
    pub enum PushForm {
        /// The opcode that pushes the empty item.
        Empty,
        /// The opcodes that push a single small positive byte.
        SmallNumber,
        /// The opcode that pushes the single negative-one byte.
        NegativeOne,
        /// The opcodes whose own byte states the payload's width.
        Direct,
        /// The opcode followed by a one-byte width.
        ExtendedOneByteWidth,
        /// The opcode followed by a two-byte width.
        ExtendedTwoByteWidth,
        /// The opcode followed by a four-byte width.
        ExtendedFourByteWidth,
    }
}

/// How a form's opcode byte relates to the payload it pushes.
///
/// Exhaustive on purpose: every reviewed form carries its payload in
/// exactly one of these four ways, and a new form cannot be admitted
/// without saying which.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PushOpcodeMapping {
    /// The opcode carries the whole payload, which is exactly these
    /// bytes.
    LiteralPayload(Vec<u8>),
    /// The opcode carries a one-byte payload whose value is the opcode
    /// byte less this offset.
    NumericPayload {
        /// The offset between the opcode byte and the value it pushes.
        offset: u8,
    },
    /// The opcode byte is the payload's width, and the payload follows
    /// it.
    WidthInOpcode,
    /// A width follows the opcode in this many bytes, and the payload
    /// follows the width.
    WidthPrefix {
        /// How many bytes the width occupies.
        bytes: NonZeroUsize,
        /// The order of those bytes.
        byte_order: ByteOrder,
    },
}

census_enum! {
    /// Why a push is not one the target accepts in its canonical form.
    #[non_exhaustive]
    pub enum PushDefect {
        /// The encoded push ends before its declared width does.
        Truncated,
        /// The payload is wider than the largest literal the target admits.
        Oversized,
        /// The payload is carried in a form that is not its unique minimal
        /// one.
        NonMinimal,
    }
}

/// Which rule refuses a defective push.
///
/// The distinction is the point of the type. A consensus refusal makes
/// a spend invalid on every node; a relay-policy refusal leaves the
/// spend valid and stops nodes forwarding it. Collapsing them would
/// make an unrelayable program look invalid and an invalid one look
/// merely unusual.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PushEnforcement {
    /// The target's own validity rules refuse it.
    Consensus,
    /// A node's standardness rules refuse to relay it.
    RelayPolicy,
}

/// A test a payload either meets or does not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PushPayloadPredicate {
    /// The payload is exactly this many bytes wide.
    ExactWidth(usize),
    /// The payload is one byte whose value lies in this inclusive
    /// range.
    SingleByteInRange {
        /// The smallest matching byte.
        first: u8,
        /// The largest matching byte.
        last: u8,
    },
    /// The payload is at most this many bytes wide.
    WidthAtMost(usize),
}

impl PushPayloadPredicate {
    /// Whether `payload` meets this test.
    #[must_use]
    pub fn matches(self, payload: &[u8]) -> bool {
        match self {
            Self::ExactWidth(width) => payload.len() == width,
            Self::SingleByteInRange { first, last } => {
                matches!(payload, [only] if (first..=last).contains(only))
            }
            Self::WidthAtMost(width) => payload.len() <= width,
        }
    }
}

/// One step of the reviewed minimal-form rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PushMinimalityStep {
    predicate: PushPayloadPredicate,
    form: PushForm,
}

impl PushMinimalityStep {
    /// States that a payload meeting `predicate` is minimally carried
    /// by `form`.
    #[must_use]
    pub const fn new(predicate: PushPayloadPredicate, form: PushForm) -> Self {
        Self { predicate, form }
    }

    /// The test this step applies.
    #[must_use]
    pub const fn predicate(self) -> PushPayloadPredicate {
        self.predicate
    }

    /// The form a payload meeting the test is carried by.
    #[must_use]
    pub const fn form(self) -> PushForm {
        self.form
    }
}

/// The complete typed contract of one push form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PushFormSpec {
    form: PushForm,
    first_opcode: u8,
    last_opcode: u8,
    mapping: PushOpcodeMapping,
    minimum_payload_bytes: usize,
    maximum_payload_bytes: usize,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

/// The parts of one push form, gathered for construction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PushFormParts {
    /// The form's identity.
    pub form: PushForm,
    /// The lowest opcode byte the form occupies.
    pub first_opcode: u8,
    /// The highest opcode byte the form occupies.
    pub last_opcode: u8,
    /// How the opcode relates to the payload.
    pub mapping: PushOpcodeMapping,
    /// The narrowest payload the form can carry.
    pub minimum_payload_bytes: usize,
    /// The widest payload the form's own width field can express,
    /// before the target's literal bound is applied.
    pub maximum_payload_bytes: usize,
    /// The evidence a deployment must produce for the form.
    pub evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl PushFormSpec {
    /// States the contract of one push form.
    #[must_use]
    pub fn new(parts: PushFormParts) -> Self {
        Self {
            form: parts.form,
            first_opcode: parts.first_opcode,
            last_opcode: parts.last_opcode,
            mapping: parts.mapping,
            minimum_payload_bytes: parts.minimum_payload_bytes,
            maximum_payload_bytes: parts.maximum_payload_bytes,
            evidence: parts.evidence,
        }
    }

    /// The form's identity.
    #[must_use]
    pub const fn form(&self) -> PushForm {
        self.form
    }

    /// The lowest opcode byte the form occupies.
    #[must_use]
    pub const fn first_opcode(&self) -> u8 {
        self.first_opcode
    }

    /// The highest opcode byte the form occupies.
    #[must_use]
    pub const fn last_opcode(&self) -> u8 {
        self.last_opcode
    }

    /// How the opcode relates to the payload.
    #[must_use]
    pub const fn mapping(&self) -> &PushOpcodeMapping {
        &self.mapping
    }

    /// The narrowest payload the form can carry.
    #[must_use]
    pub const fn minimum_payload_bytes(&self) -> usize {
        self.minimum_payload_bytes
    }

    /// The widest payload the form's own width field can express.
    #[must_use]
    pub const fn maximum_payload_bytes(&self) -> usize {
        self.maximum_payload_bytes
    }

    /// The evidence a deployment must produce for this form.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// Whether the form occupies `opcode`.
    #[must_use]
    pub const fn occupies(&self, opcode: u8) -> bool {
        self.first_opcode <= opcode && opcode <= self.last_opcode
    }

    /// Whether the form's own width field can express `width`.
    #[must_use]
    pub const fn admits_width(&self, width: usize) -> bool {
        self.minimum_payload_bytes <= width && width <= self.maximum_payload_bytes
    }

    /// How many bytes of width follow the opcode under this form.
    #[must_use]
    pub const fn width_prefix_bytes(&self) -> usize {
        match self.mapping {
            PushOpcodeMapping::WidthPrefix { bytes, .. } => bytes.get(),
            PushOpcodeMapping::LiteralPayload(_)
            | PushOpcodeMapping::NumericPayload { .. }
            | PushOpcodeMapping::WidthInOpcode => 0,
        }
    }

    /// The order of this form's width prefix, for a form that has one.
    #[must_use]
    pub const fn width_prefix_order(&self) -> Option<ByteOrder> {
        match self.mapping {
            PushOpcodeMapping::WidthPrefix { byte_order, .. } => Some(byte_order),
            PushOpcodeMapping::LiteralPayload(_)
            | PushOpcodeMapping::NumericPayload { .. }
            | PushOpcodeMapping::WidthInOpcode => None,
        }
    }

    /// The opcode byte that carries `payload` under this form.
    ///
    /// Returns `None` when the form cannot carry that payload at all,
    /// which is the same question a serializer would otherwise answer
    /// by writing the byte down itself.
    #[must_use]
    pub fn opcode_for(&self, payload: &[u8]) -> Option<u8> {
        if !self.admits_width(payload.len()) {
            return None;
        }
        match &self.mapping {
            PushOpcodeMapping::LiteralPayload(literal) => {
                (literal.as_slice() == payload).then_some(self.first_opcode)
            }
            PushOpcodeMapping::NumericPayload { offset } => match payload {
                [only] => only
                    .checked_add(*offset)
                    .filter(|byte| self.occupies(*byte)),
                _ => None,
            },
            PushOpcodeMapping::WidthInOpcode => {
                let byte = u8::try_from(payload.len()).ok()?;
                self.occupies(byte).then_some(byte)
            }
            PushOpcodeMapping::WidthPrefix { .. } => Some(self.first_opcode),
        }
    }

    /// The payload `opcode` carries by itself, for a form that needs no
    /// following bytes.
    ///
    /// Returns `None` for a form whose payload comes from the script
    /// after the opcode, and for an opcode the form does not occupy.
    #[must_use]
    pub fn payload_for(&self, opcode: u8) -> Option<Vec<u8>> {
        if !self.occupies(opcode) {
            return None;
        }
        match &self.mapping {
            PushOpcodeMapping::LiteralPayload(literal) => Some(literal.clone()),
            PushOpcodeMapping::NumericPayload { offset } => {
                opcode.checked_sub(*offset).map(|value| vec![value])
            }
            PushOpcodeMapping::WidthInOpcode | PushOpcodeMapping::WidthPrefix { .. } => None,
        }
    }

    /// The payload width `opcode` states by itself, for a form that
    /// carries its width in the opcode.
    #[must_use]
    pub fn width_in_opcode(&self, opcode: u8) -> Option<usize> {
        match self.mapping {
            PushOpcodeMapping::WidthInOpcode if self.occupies(opcode) => Some(usize::from(opcode)),
            _ => None,
        }
    }
}

/// The complete reviewed push contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PushContract {
    forms: BTreeMap<PushForm, PushFormSpec>,
    maximum_payload_bytes: usize,
    minimality: Vec<PushMinimalityStep>,
    enforcement: BTreeMap<PushDefect, PushEnforcement>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl PushContract {
    /// States the push contract.
    #[must_use]
    pub fn new(
        forms: impl IntoIterator<Item = PushFormSpec>,
        maximum_payload_bytes: usize,
        minimality: impl IntoIterator<Item = PushMinimalityStep>,
        enforcement: impl IntoIterator<Item = (PushDefect, PushEnforcement)>,
        evidence: impl IntoIterator<Item = TargetEvidenceRequirementId>,
    ) -> Self {
        Self {
            forms: forms.into_iter().map(|spec| (spec.form(), spec)).collect(),
            maximum_payload_bytes,
            minimality: minimality.into_iter().collect(),
            enforcement: enforcement.into_iter().collect(),
            evidence: evidence.into_iter().collect(),
        }
    }

    /// The declared forms, in stable identity order.
    #[must_use]
    pub const fn forms(&self) -> &BTreeMap<PushForm, PushFormSpec> {
        &self.forms
    }

    /// The contract of one form.
    #[must_use]
    pub fn form(&self, form: PushForm) -> Option<&PushFormSpec> {
        self.forms.get(&form)
    }

    /// The largest literal the target accepts on its stack.
    #[must_use]
    pub const fn maximum_payload_bytes(&self) -> usize {
        self.maximum_payload_bytes
    }

    /// The ordered rule selecting a payload's minimal form.
    ///
    /// The order is the reviewed fact, not a convenience: the target
    /// applies these tests in sequence and takes the first that
    /// matches, so a one-byte payload of five is carried by the small
    /// number form rather than by the direct push that would also
    /// admit its width.
    #[must_use]
    pub fn minimality(&self) -> &[PushMinimalityStep] {
        &self.minimality
    }

    /// Which rule refuses each defect.
    #[must_use]
    pub const fn enforcement(&self) -> &BTreeMap<PushDefect, PushEnforcement> {
        &self.enforcement
    }

    /// The evidence a deployment must produce for the push rules.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// The form occupying `opcode`, if any push form does.
    #[must_use]
    pub fn form_for_opcode(&self, opcode: u8) -> Option<&PushFormSpec> {
        self.forms.values().find(|spec| spec.occupies(opcode))
    }

    /// The unique minimal form of `payload`.
    ///
    /// # Errors
    ///
    /// [`PushDefect::Oversized`] when the payload exceeds the largest
    /// literal the target accepts, and [`PushDefect::NonMinimal`] when
    /// the rule names no form for it, so that every form carrying it
    /// would be refused.
    pub fn minimal_form(&self, payload: &[u8]) -> Result<PushForm, PushDefect> {
        if payload.len() > self.maximum_payload_bytes {
            return Err(PushDefect::Oversized);
        }
        self.minimality
            .iter()
            .find(|step| step.predicate().matches(payload))
            .map(|step| step.form())
            .ok_or(PushDefect::NonMinimal)
    }

    /// Every opcode byte the push forms occupy.
    #[must_use]
    pub fn occupied_opcodes(&self) -> BTreeSet<u8> {
        self.forms
            .values()
            .flat_map(|spec| spec.first_opcode()..=spec.last_opcode())
            .collect()
    }

    /// The first way the contract fails to describe usable push rules.
    ///
    /// Reported one defect at a time by the definition validator, which
    /// gathers them alongside every other diagnostic.
    #[must_use]
    pub fn defects(&self) -> Vec<PushContractDefect> {
        let mut defects = Vec::new();

        for form in PushForm::ALL {
            if !self.forms.contains_key(form) {
                defects.push(PushContractDefect::MissingForm(*form));
            }
        }

        // A form filed under another identity is unconstructible: the
        // map is keyed by what each specification declares, so there is
        // no defect to report and no branch that could report one.
        let mut claimed: BTreeMap<u8, PushForm> = BTreeMap::new();
        for (key, spec) in &self.forms {
            if spec.first_opcode() > spec.last_opcode()
                || spec.minimum_payload_bytes() > spec.maximum_payload_bytes()
            {
                defects.push(PushContractDefect::IncoherentForm(*key));
            }
            for opcode in spec.first_opcode()..=spec.last_opcode() {
                if claimed.insert(opcode, *key).is_some() {
                    defects.push(PushContractDefect::DuplicateOpcode(opcode));
                }
            }
            if spec.evidence().is_empty() {
                defects.push(PushContractDefect::MissingFormEvidence(*key));
            }
        }

        for defect in PushDefect::ALL {
            if !self.enforcement.contains_key(defect) {
                defects.push(PushContractDefect::MissingEnforcement(*defect));
            }
        }

        defects
    }
}

/// Why a push contract does not describe usable rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PushContractDefect {
    /// A reviewed form has no contract.
    MissingForm(PushForm),
    /// A form states a span or a width range that admits nothing.
    IncoherentForm(PushForm),
    /// Two forms claim the same opcode byte, so a decoder could not say
    /// which one a script used.
    DuplicateOpcode(u8),
    /// A form names no evidence, so its rule rests on this crate alone.
    MissingFormEvidence(PushForm),
    /// A defect has no stated enforcement, so a consumer could not tell
    /// an invalid program from an unrelayable one.
    MissingEnforcement(PushDefect),
}

// ---------------------------------------------------------------
// The reviewed contract.
// ---------------------------------------------------------------

/// The opcode byte pushing the empty item.
const EMPTY_OPCODE: u8 = 0x00;

/// The lowest opcode byte whose own value is a payload width.
const FIRST_DIRECT_OPCODE: u8 = 0x01;

/// The highest opcode byte whose own value is a payload width.
const LAST_DIRECT_OPCODE: u8 = 0x4b;

/// The opcode byte introducing a one-byte width.
const ONE_BYTE_WIDTH_OPCODE: u8 = 0x4c;

/// The opcode byte introducing a two-byte width.
const TWO_BYTE_WIDTH_OPCODE: u8 = 0x4d;

/// The opcode byte introducing a four-byte width.
const FOUR_BYTE_WIDTH_OPCODE: u8 = 0x4e;

/// The opcode byte pushing the single negative-one payload.
const NEGATIVE_ONE_OPCODE: u8 = 0x4f;

/// The lowest opcode byte pushing a small positive value.
const FIRST_SMALL_NUMBER_OPCODE: u8 = 0x51;

/// The highest opcode byte pushing a small positive value.
const LAST_SMALL_NUMBER_OPCODE: u8 = 0x60;

/// The offset between a small-number opcode and the value it pushes.
const SMALL_NUMBER_OFFSET: u8 = 0x50;

/// The single byte the negative-one opcode pushes.
const NEGATIVE_ONE_PAYLOAD: u8 = 0x81;

/// The widest payload a one-byte width can state.
const ONE_BYTE_WIDTH_MAXIMUM: usize = 255;

/// The widest payload a two-byte width can state.
const TWO_BYTE_WIDTH_MAXIMUM: usize = 65_535;

/// The widest payload a four-byte width can state.
const FOUR_BYTE_WIDTH_MAXIMUM: usize = 4_294_967_295;

/// A width in bytes, for the form declarations.
const fn width(bytes: usize) -> NonZeroUsize {
    NonZeroUsize::new(bytes).expect("a reviewed width prefix is never zero")
}

/// Builds one push form, evidenced by the push requirement.
fn form(
    form: PushForm,
    first_opcode: u8,
    last_opcode: u8,
    mapping: PushOpcodeMapping,
    minimum_payload_bytes: usize,
    maximum_payload_bytes: usize,
) -> PushFormSpec {
    PushFormSpec::new(PushFormParts {
        form,
        first_opcode,
        last_opcode,
        mapping,
        minimum_payload_bytes,
        maximum_payload_bytes,
        evidence: std::iter::once(TargetEvidenceRequirementId::PushEncodingSemantics).collect(),
    })
}

/// The reviewed push forms.
fn reviewed_forms() -> Vec<PushFormSpec> {
    use ByteOrder::LittleEndian as LE;
    use PushOpcodeMapping as M;

    vec![
        form(
            PushForm::Empty,
            EMPTY_OPCODE,
            EMPTY_OPCODE,
            M::LiteralPayload(Vec::new()),
            0,
            0,
        ),
        form(
            PushForm::Direct,
            FIRST_DIRECT_OPCODE,
            LAST_DIRECT_OPCODE,
            M::WidthInOpcode,
            usize::from(FIRST_DIRECT_OPCODE),
            usize::from(LAST_DIRECT_OPCODE),
        ),
        // The three extended forms differ only in how wide their width
        // is, and the width is read least significant byte first.
        form(
            PushForm::ExtendedOneByteWidth,
            ONE_BYTE_WIDTH_OPCODE,
            ONE_BYTE_WIDTH_OPCODE,
            M::WidthPrefix {
                bytes: width(1),
                byte_order: LE,
            },
            0,
            ONE_BYTE_WIDTH_MAXIMUM,
        ),
        form(
            PushForm::ExtendedTwoByteWidth,
            TWO_BYTE_WIDTH_OPCODE,
            TWO_BYTE_WIDTH_OPCODE,
            M::WidthPrefix {
                bytes: width(2),
                byte_order: LE,
            },
            0,
            TWO_BYTE_WIDTH_MAXIMUM,
        ),
        // Declared because the target decodes it, not because a
        // first-party program ever emits one: no payload the target
        // accepts on its stack is wide enough for this form to be the
        // minimal one, so every use of it is refused as nonminimal or
        // as oversized.
        form(
            PushForm::ExtendedFourByteWidth,
            FOUR_BYTE_WIDTH_OPCODE,
            FOUR_BYTE_WIDTH_OPCODE,
            M::WidthPrefix {
                bytes: width(4),
                byte_order: LE,
            },
            0,
            FOUR_BYTE_WIDTH_MAXIMUM,
        ),
        form(
            PushForm::NegativeOne,
            NEGATIVE_ONE_OPCODE,
            NEGATIVE_ONE_OPCODE,
            M::LiteralPayload(vec![NEGATIVE_ONE_PAYLOAD]),
            1,
            1,
        ),
        form(
            PushForm::SmallNumber,
            FIRST_SMALL_NUMBER_OPCODE,
            LAST_SMALL_NUMBER_OPCODE,
            M::NumericPayload {
                offset: SMALL_NUMBER_OFFSET,
            },
            1,
            1,
        ),
    ]
}

/// The reviewed minimal-form rule, in the order the target applies it.
fn reviewed_minimality() -> Vec<PushMinimalityStep> {
    use PushPayloadPredicate as P;

    vec![
        PushMinimalityStep::new(P::ExactWidth(0), PushForm::Empty),
        PushMinimalityStep::new(
            P::SingleByteInRange {
                first: 1,
                last: LAST_SMALL_NUMBER_OPCODE - SMALL_NUMBER_OFFSET,
            },
            PushForm::SmallNumber,
        ),
        PushMinimalityStep::new(
            P::SingleByteInRange {
                first: NEGATIVE_ONE_PAYLOAD,
                last: NEGATIVE_ONE_PAYLOAD,
            },
            PushForm::NegativeOne,
        ),
        PushMinimalityStep::new(
            P::WidthAtMost(usize::from(LAST_DIRECT_OPCODE)),
            PushForm::Direct,
        ),
        PushMinimalityStep::new(
            P::WidthAtMost(ONE_BYTE_WIDTH_MAXIMUM),
            PushForm::ExtendedOneByteWidth,
        ),
        PushMinimalityStep::new(
            P::WidthAtMost(TWO_BYTE_WIDTH_MAXIMUM),
            PushForm::ExtendedTwoByteWidth,
        ),
    ]
}

/// Builds the reviewed push contract.
///
/// The maximum literal size and the well-formedness of an encoded push
/// are consensus rules; minimal encoding is a relay rule. That split is
/// a reviewed target fact and it is the reason the enforcement census
/// exists at all.
pub(crate) fn reviewed_pushes() -> PushContract {
    PushContract::new(
        reviewed_forms(),
        MAX_STACK_ELEMENT_BYTES,
        reviewed_minimality(),
        [
            (PushDefect::Truncated, PushEnforcement::Consensus),
            (PushDefect::Oversized, PushEnforcement::Consensus),
            (PushDefect::NonMinimal, PushEnforcement::RelayPolicy),
        ],
        [TargetEvidenceRequirementId::PushEncodingSemantics],
    )
}
