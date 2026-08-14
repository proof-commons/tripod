//! The generic native fixture language.
//!
//! A fixture describes a generic target execution: exact script bytes,
//! an exact initial stack, the transaction context an introspection
//! primitive needs, and the outcome the reviewed contract says the
//! target must produce.
//!
//! # Nothing here is the attestation contract
//!
//! A fixture carries no operation, no receipt, no state, no owner, no
//! family position, no root, and no relation. It is a target
//! transaction and a target stack, and the vocabulary it uses to
//! describe them is the target's own (Guide-9 §1.9, §10.1).
//!
//! # Expectations come from the contract, never from the executor
//!
//! [`ExpectedPrimitiveOutcome`] is authored against the reviewed typed
//! contract and independently reviewed byte vectors. It is never
//! derived from what an executor returned: a harness that adopts the
//! observation as the expectation tests that the executor is
//! self-consistent, which every executor is.
//!
//! # Exact bytes, produced typed
//!
//! A fixture's script is produced by encoding a typed
//! [`TapscriptProgram`] and is then retained as exact bytes. The typed
//! program is how the bytes come to exist; the bytes are what the
//! executor is asked about, because a re-encoding at execution time
//! would let a serializer change what was tested.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use tapscript::{StackItem, TapscriptProgram};
use target_elements::{OpcodeId, ReviewedElementsTapscriptDefinition, ValidatedDevelopmentBinding};

use crate::error::NativeConformanceError;
use crate::protocol::{ObservedFailureClass, WireExecutionDomain};
use crate::vocabulary::{opcode_from_name, opcode_name};

/// Which dimension of the target contract a case exercises.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum NativeCaseGroup {
    /// That scripts execute in the reviewed domain.
    ExecutionDomain,
    /// That the reviewed leaf version selects that domain.
    LeafVersion,
    /// That reviewed primitive bytes decode as reviewed primitives.
    InstructionEncoding,
    /// That literal pushes take the reviewed forms.
    PushEncoding,
    /// Input introspection.
    InputIntrospection,
    /// Output introspection.
    OutputIntrospection,
    /// Whole-transaction introspection.
    TransactionIntrospection,
    /// Signed fixed-width arithmetic.
    Arithmetic,
    /// Signed fixed-width comparison.
    Comparison,
    /// Conversions between numeric forms.
    Conversion,
    /// The streaming hash primitives.
    StreamingHash,
    /// The elliptic-curve primitives.
    EllipticCurve,
    /// Signature verification.
    Signature,
    /// What the sighash commits to.
    Sighash,
    /// Relative timelocks.
    RelativeTimelock,
    /// Confidential values and their conservation.
    ConfidentialValue,
    /// Issuance and reissuance introspection.
    Issuance,
    /// Resource accounting.
    Resource,
}

impl NativeCaseGroup {
    /// The complete census of fixture groups.
    pub const ALL: &'static [Self] = &[
        Self::ExecutionDomain,
        Self::LeafVersion,
        Self::InstructionEncoding,
        Self::PushEncoding,
        Self::InputIntrospection,
        Self::OutputIntrospection,
        Self::TransactionIntrospection,
        Self::Arithmetic,
        Self::Comparison,
        Self::Conversion,
        Self::StreamingHash,
        Self::EllipticCurve,
        Self::Signature,
        Self::Sighash,
        Self::RelativeTimelock,
        Self::ConfidentialValue,
        Self::Issuance,
        Self::Resource,
    ];

    /// The spelling this group travels under.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::ExecutionDomain => "execution_domain",
            Self::LeafVersion => "leaf_version",
            Self::InstructionEncoding => "instruction_encoding",
            Self::PushEncoding => "push_encoding",
            Self::InputIntrospection => "input_introspection",
            Self::OutputIntrospection => "output_introspection",
            Self::TransactionIntrospection => "transaction_introspection",
            Self::Arithmetic => "arithmetic",
            Self::Comparison => "comparison",
            Self::Conversion => "conversion",
            Self::StreamingHash => "streaming_hash",
            Self::EllipticCurve => "elliptic_curve",
            Self::Signature => "signature",
            Self::Sighash => "sighash",
            Self::RelativeTimelock => "relative_timelock",
            Self::ConfidentialValue => "confidential_value",
            Self::Issuance => "issuance",
            Self::Resource => "resource",
        }
    }
}

/// The complete typed key of one fixture.
///
/// A key, not a digest: the identity of a case is the group it belongs
/// to, the primitive it exercises where it exercises one, and its
/// ordinal within that pair. The ordinal is stable only inside the
/// declared fixture set and is not a public semantic identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NativeCaseId {
    group: NativeCaseGroup,
    opcode: Option<OpcodeId>,
    ordinal: u32,
}

impl NativeCaseId {
    /// Names one case.
    #[must_use]
    pub const fn new(group: NativeCaseGroup, opcode: Option<OpcodeId>, ordinal: u32) -> Self {
        Self {
            group,
            opcode,
            ordinal,
        }
    }

    /// The dimension the case exercises.
    #[must_use]
    pub const fn group(self) -> NativeCaseGroup {
        self.group
    }

    /// The primitive the case exercises, where it exercises one.
    #[must_use]
    pub const fn opcode(self) -> Option<OpcodeId> {
        self.opcode
    }

    /// The case's ordinal within its group and primitive.
    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.ordinal
    }
}

impl fmt::Display for NativeCaseId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let opcode = self
            .opcode
            .and_then(opcode_name)
            .unwrap_or("no-reviewed-primitive");
        write!(
            formatter,
            "{}/{opcode}/{}",
            self.group.wire_name(),
            self.ordinal
        )
    }
}

/// The wire form of a case identity.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireCaseId {
    group: NativeCaseGroup,
    opcode: Option<String>,
    ordinal: u32,
}

impl Serialize for NativeCaseId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let opcode = match self.opcode {
            None => None,
            Some(id) => Some(
                opcode_name(id)
                    .ok_or_else(|| {
                        serde::ser::Error::custom("a case names a primitive with no wire spelling")
                    })?
                    .to_owned(),
            ),
        };
        WireCaseId {
            group: self.group,
            opcode,
            ordinal: self.ordinal,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NativeCaseId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = WireCaseId::deserialize(deserializer)?;
        let opcode = match wire.opcode {
            None => None,
            Some(name) => Some(opcode_from_name(&name).ok_or_else(|| {
                serde::de::Error::custom("a case names a primitive with no wire spelling")
            })?),
        };
        Ok(Self {
            group: wire.group,
            opcode,
            ordinal: wire.ordinal,
        })
    }
}

/// What the reviewed contract says the target must do with one case.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ExpectedPrimitiveOutcome {
    /// The target accepts, leaving exactly these stacks.
    Accept {
        /// The exact final main stack.
        final_stack: Vec<Vec<u8>>,
        /// The exact final alternate stack.
        final_altstack: Vec<Vec<u8>>,
    },
    /// The target rejects, in exactly this class.
    Reject {
        /// The failure class the contract declares.
        class: ObservedFailureClass,
    },
}

/// One input of a fixture transaction.
///
/// Target-owned fields only. There is no protocol family position, no
/// root, and no owner here, and none may be added: a fixture describes a
/// target transaction, and a field naming an attestation-contract
/// object would make it describe something else.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureInput {
    /// The spent outpoint's transaction identifier.
    pub outpoint_txid: [u8; 32],
    /// The spent outpoint's index.
    pub outpoint_index: u32,
    /// The spent output's asset field, prefix included.
    pub spent_asset: Vec<u8>,
    /// The spent output's value field, prefix included.
    pub spent_value: Vec<u8>,
    /// The spent output's program.
    pub spent_program: Vec<u8>,
    /// The input's sequence number.
    pub sequence: u32,
    /// The input's issuance fields, where it carries an issuance.
    pub issuance: Option<FixtureIssuance>,
    /// The input's initial witness stack.
    pub witness: Vec<Vec<u8>>,
}

/// The issuance fields of one input.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureIssuance {
    /// The issued asset amount field, prefix included.
    pub asset_amount: Vec<u8>,
    /// The inflation-keys amount field, prefix included.
    pub inflation_keys_amount: Vec<u8>,
    /// The issuance entropy.
    pub entropy: [u8; 32],
    /// The issuance blinding nonce.
    pub blinding_nonce: [u8; 32],
}

/// One output of a fixture transaction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureOutput {
    /// The asset field, prefix included.
    pub asset: Vec<u8>,
    /// The value field, prefix included.
    pub value: Vec<u8>,
    /// The nonce field, prefix included.
    pub nonce: Vec<u8>,
    /// The output program.
    pub program: Vec<u8>,
}

/// The script-path data the executor spends through.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureScriptPath {
    /// The leaf version byte.
    pub leaf_version: u8,
    /// The exact leaf script bytes.
    pub script: Vec<u8>,
    /// The control data, where the executor requires it.
    pub control: Vec<u8>,
}

/// The generic transaction context an introspection primitive reads.
///
/// Input and output counts are not fields: they are the lengths of the
/// two vectors, and a separately stated count could disagree with them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrimitiveExecutionContext {
    /// The transaction version.
    pub version: u32,
    /// The transaction locktime.
    pub locktime: u32,
    /// Which input is being validated.
    pub current_input_index: u32,
    /// The transaction's inputs.
    pub inputs: Vec<FixtureInput>,
    /// The transaction's outputs.
    pub outputs: Vec<FixtureOutput>,
    /// The script path being spent.
    pub script_path: FixtureScriptPath,
}

/// One generic target execution, and the outcome the contract requires.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrimitiveFixture {
    case: NativeCaseId,
    target_contract_version: u32,
    network_id: [u8; 32],
    genesis_id: [u8; 32],
    execution_domain: WireExecutionDomain,
    leaf_version: u8,
    script: Vec<u8>,
    initial_stack: Vec<Vec<u8>>,
    context: Option<PrimitiveExecutionContext>,
    expected: ExpectedPrimitiveOutcome,
}

impl PrimitiveFixture {
    /// States one fixture against the reviewed contract and one
    /// development binding.
    ///
    /// The contract revision, execution domain, leaf version, and the
    /// two network identifiers are read from those two values rather
    /// than accepted from the caller, so a fixture cannot claim to be
    /// stated against a contract or a network it was not.
    ///
    /// # Errors
    ///
    /// [`NativeConformanceError::TargetContractMismatch`] when the
    /// reviewed contract's execution domain has no wire spelling, which
    /// means the domain is one this harness has never seen.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        binding: &ValidatedDevelopmentBinding,
        case: NativeCaseId,
        program: &TapscriptProgram,
        initial_stack: &[StackItem],
        context: Option<PrimitiveExecutionContext>,
        expected: ExpectedPrimitiveOutcome,
    ) -> Result<Self, NativeConformanceError> {
        let definition = target.definition();
        let domain = WireExecutionDomain::of(definition.execution_domain())
            .ok_or(NativeConformanceError::TargetContractMismatch)?;
        Ok(Self {
            case,
            target_contract_version: definition.version().get(),
            network_id: binding.binding().network_id(),
            genesis_id: binding.binding().genesis_id(),
            execution_domain: domain,
            leaf_version: definition.leaf_version().get(),
            // Exact bytes, produced once through the typed program.
            script: program.encode(target),
            initial_stack: initial_stack
                .iter()
                .map(|item| item.bytes().to_vec())
                .collect(),
            context,
            expected,
        })
    }

    /// The case this fixture answers for.
    #[must_use]
    pub const fn case(&self) -> NativeCaseId {
        self.case
    }

    /// The contract revision the fixture is stated against.
    #[must_use]
    pub const fn target_contract_version(&self) -> u32 {
        self.target_contract_version
    }

    /// The network the fixture is stated against.
    #[must_use]
    pub const fn network_id(&self) -> [u8; 32] {
        self.network_id
    }

    /// The genesis identifier the fixture is stated against.
    #[must_use]
    pub const fn genesis_id(&self) -> [u8; 32] {
        self.genesis_id
    }

    /// The execution domain the fixture is stated against.
    #[must_use]
    pub const fn execution_domain(&self) -> WireExecutionDomain {
        self.execution_domain
    }

    /// The leaf version the fixture is stated against.
    #[must_use]
    pub const fn leaf_version(&self) -> u8 {
        self.leaf_version
    }

    /// The exact script bytes.
    #[must_use]
    pub fn script(&self) -> &[u8] {
        &self.script
    }

    /// The exact initial stack.
    #[must_use]
    pub fn initial_stack(&self) -> &[Vec<u8>] {
        &self.initial_stack
    }

    /// The transaction context, where the case needs one.
    #[must_use]
    pub const fn context(&self) -> Option<&PrimitiveExecutionContext> {
        self.context.as_ref()
    }

    /// What the reviewed contract requires the target to do.
    #[must_use]
    pub const fn expected(&self) -> &ExpectedPrimitiveOutcome {
        &self.expected
    }
}

/// The canonical fixture census.
///
/// Sorted by typed case identity, so the declaration order of the
/// fixtures affects neither the request order, the report's case order,
/// nor the summary counts. A repeated identity is refused rather than
/// resolved: two fixtures under one key would make "the result for this
/// case" ambiguous, and the ambiguity would be settled by whichever one
/// was declared last.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PrimitiveFixtureSet {
    fixtures: BTreeMap<NativeCaseId, PrimitiveFixture>,
}

impl PrimitiveFixtureSet {
    /// Collects fixtures into the canonical census.
    ///
    /// # Errors
    ///
    /// [`NativeConformanceError::DuplicateFixtureCase`] when two
    /// fixtures declare one case identity.
    pub fn new(
        fixtures: impl IntoIterator<Item = PrimitiveFixture>,
    ) -> Result<Self, NativeConformanceError> {
        let mut census = BTreeMap::new();
        for fixture in fixtures {
            let case = fixture.case();
            if census.insert(case, fixture).is_some() {
                return Err(NativeConformanceError::DuplicateFixtureCase(case));
            }
        }
        Ok(Self { fixtures: census })
    }

    /// The fixtures, in canonical case order.
    pub fn iter(&self) -> impl Iterator<Item = &PrimitiveFixture> {
        self.fixtures.values()
    }

    /// How many fixtures the census holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.fixtures.len()
    }

    /// Whether the census is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fixtures.is_empty()
    }

    /// Whether the census holds one case.
    #[must_use]
    pub fn contains(&self, case: NativeCaseId) -> bool {
        self.fixtures.contains_key(&case)
    }
}

impl<'a> IntoIterator for &'a PrimitiveFixtureSet {
    type Item = &'a PrimitiveFixture;
    type IntoIter = std::collections::btree_map::Values<'a, NativeCaseId, PrimitiveFixture>;

    fn into_iter(self) -> Self::IntoIter {
        self.fixtures.values()
    }
}

/// The canonical fixture census of this repository.
///
/// Empty: no primitive fixture has been authored yet. An empty census is
/// stated rather than filled with something plausible, because a
/// fixture that has not been reviewed against the target contract is not
/// evidence, and the report is required to say so — a run over this
/// census cannot satisfy the Guide-9 evidence plan and does not pretend
/// to.
///
/// # Errors
///
/// [`NativeConformanceError::DuplicateFixtureCase`] once the census
/// holds fixtures and two of them declare one case identity.
pub fn canonical_fixture_set(
    _target: &ReviewedElementsTapscriptDefinition,
    _binding: &ValidatedDevelopmentBinding,
) -> Result<PrimitiveFixtureSet, NativeConformanceError> {
    PrimitiveFixtureSet::new([])
}
