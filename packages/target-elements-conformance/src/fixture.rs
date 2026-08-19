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

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use tapscript::{StackItem, TapscriptProgram};
use target_elements::{OpcodeId, ReviewedDevelopmentBinding, ReviewedElementsTapscriptDefinition};

use crate::claim::{NativeEvidenceClaim, claims_of};
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
    /// The ordinary stack operations.
    StackRearrangement,
    /// Byte-string concatenation, slicing, width, and bitwise logic.
    ByteString,
    /// Equality and Boolean verification.
    Verification,
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
        Self::StackRearrangement,
        Self::ByteString,
        Self::Verification,
    ];

    /// The spelling this group travels under.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::StackRearrangement => "stack_rearrangement",
            Self::ByteString => "byte_string",
            Self::Verification => "verification",
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
///
/// # A verdict, and the classes the contract admits for it
///
/// The observation a target-native executor can actually make is whether
/// the spend was valid, and -- where the target distinguishes them --
/// why it was not. The failure side is therefore a *set*: the target
/// reports one reason for several reviewed causes, and a fixture naming
/// one of them would fail an honest executor over a distinction the
/// target does not draw. The set is the contract's admissible causes for
/// the case, and an observation passes when it is one of them.
///
/// # The stacks are static, and say so
///
/// A stack here is what the reviewed contract and the abstract validator
/// establish, not what a node reports: a validating node exposes no
/// interpreter stack at all. The expected stacks are carried because
/// they are the fixture's complete statement of the case, and because an
/// executor that *does* report a stack is then compared against them --
/// never because a run without them is incomplete (Guide-9 §15.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ExpectedPrimitiveOutcome {
    /// The target accepts the spend.
    Accept {
        /// The exact final main stack, deepest item first, where the
        /// contract fixes one.
        static_final_stack: Option<Vec<Vec<u8>>>,
        /// The exact final alternate stack, deepest item first.
        static_final_altstack: Option<Vec<Vec<u8>>>,
    },
    /// The target rejects the spend, for one of these reasons.
    Reject {
        /// The failure classes the reviewed contract admits here.
        classes: BTreeSet<ObservedFailureClass>,
        /// The exact final main stack, where the case leaves one and
        /// the contract fixes it.
        static_final_stack: Option<Vec<Vec<u8>>>,
        /// The exact final alternate stack, where the case leaves one.
        static_final_altstack: Option<Vec<Vec<u8>>>,
    },
}

impl ExpectedPrimitiveOutcome {
    /// The spend is valid, leaving exactly this stack.
    #[must_use]
    pub const fn accept(static_final_stack: Option<Vec<Vec<u8>>>) -> Self {
        Self::Accept {
            static_final_stack,
            static_final_altstack: Some(Vec::new()),
        }
    }

    /// The spend is invalid, for one of these reasons.
    #[must_use]
    pub fn reject(
        classes: impl IntoIterator<Item = ObservedFailureClass>,
        static_final_stack: Option<Vec<Vec<u8>>>,
    ) -> Self {
        Self::Reject {
            classes: classes.into_iter().collect(),
            static_final_stack,
            static_final_altstack: Some(Vec::new()),
        }
    }

    /// Whether the outcome says the spend is valid.
    #[must_use]
    pub const fn is_accepting(&self) -> bool {
        matches!(self, Self::Accept { .. })
    }

    /// The failure classes the outcome admits.
    #[must_use]
    pub fn classes(&self) -> BTreeSet<ObservedFailureClass> {
        match self {
            Self::Accept { .. } => BTreeSet::new(),
            Self::Reject { classes, .. } => classes.clone(),
        }
    }

    /// The exact final main stack, where the outcome fixes one.
    #[must_use]
    pub fn static_final_stack(&self) -> Option<&[Vec<u8>]> {
        match self {
            Self::Accept {
                static_final_stack, ..
            }
            | Self::Reject {
                static_final_stack, ..
            } => static_final_stack.as_deref(),
        }
    }

    /// The exact final alternate stack, where the outcome fixes one.
    #[must_use]
    pub fn static_final_altstack(&self) -> Option<&[Vec<u8>]> {
        match self {
            Self::Accept {
                static_final_altstack,
                ..
            }
            | Self::Reject {
                static_final_altstack,
                ..
            } => static_final_altstack.as_deref(),
        }
    }
}

/// Which rule a fixture's verdict is stated at.
///
/// The target's own validity rules and a node's relay rules refuse
/// different things: a nonminimally encoded literal is a valid spend
/// that nodes will not forward, and one verdict covering both layers
/// would make an unrelayable program look invalid. Every fixture says
/// which layer its verdict belongs to, and an executor answers at the
/// layer it validates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum EnforcementLayer {
    /// The target's own validity rules.
    Consensus,
    /// A node's relay rules, which leave the spend valid.
    RelayPolicy,
}

/// Whether a fixture's leaf version is the reviewed one.
///
/// The reviewed status travels with the byte because one case needs a
/// byte the contract has *not* reviewed. A leaf version selects the
/// semantics of everything under it, and the reviewed semantics are
/// exactly what an unreviewed byte does not select: the reviewed
/// primitives are not executed there at all. No fixture stated at the
/// reviewed version can establish that, which is why this one is not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum LeafVersionStatus {
    /// The leaf version the reviewed contract fixes.
    Reviewed,
    /// A byte the contract has not reviewed, stated so that the
    /// reviewed semantics can be observed not applying under it.
    Unreviewed,
}

/// Where a fixture's exact script bytes came from.
///
/// A fixture's bytes are normally produced by encoding a typed
/// [`TapscriptProgram`], which is what makes them reviewed bytes rather
/// than bytes somebody typed. The encoding-conformance cases are the
/// exception on purpose: a truncated push, an oversized literal, and a
/// nonminimal form are precisely the shapes the typed program refuses to
/// build, so those fixtures state their bytes directly and say so here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FixtureScriptSource {
    /// The bytes are one typed program's own encoding.
    TypedProgram,
    /// The bytes are stated directly because no typed program expresses
    /// the malformed or nonminimal form the case exercises.
    DeliberatelyMalformed,
}

/// How one observed resource figure is compared.
///
/// Exact where the fixture or the reviewed contract fixes the value;
/// recorded-only everywhere else. A figure nothing fixes is still worth
/// carrying — it is the observation §13.16 asks for — but comparing it
/// against a number this package invented would fail honest executors
/// over a value no contract states.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ResourceExpectation {
    /// The figure is fixed, and an observation must match it exactly.
    Exact(u64),
    /// The figure is observed and recorded, and not compared.
    RecordedOnly,
}

/// What one execution is expected to cost.
///
/// Two rows are exact: the script's size and the initial stack's depth
/// are fixed by the fixture itself, so a disagreement means the executor
/// ran something other than what it was handed. Everything else is
/// recorded rather than compared, because a validating node observes
/// what a transaction cost and not what an interpreter's stack did on
/// the way — and a peak this package invented would fail an honest
/// executor over a figure no contract states.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedResourceObservation {
    /// The script's size in bytes.
    pub script_bytes: ResourceExpectation,
    /// How many items the initial stack held.
    pub initial_stack_items: ResourceExpectation,
    /// The deepest the main stack became.
    pub peak_stack_items: ResourceExpectation,
    /// The deepest the alternate stack became.
    pub peak_altstack_items: ResourceExpectation,
    /// The largest stack element, in bytes.
    pub maximum_element_bytes: ResourceExpectation,
    /// How much validation budget the execution used.
    pub validation_budget_used: ResourceExpectation,
    /// The materialized transaction's weight.
    pub transaction_weight: ResourceExpectation,
}

/// One input of a fixture transaction.
///
/// Target-owned fields only. There is no protocol family position, no
/// root, and no owner here, and none may be added: a fixture describes a
/// target transaction, and a field naming an attestation-contract
/// object would make it describe something else.
///
/// # What the executor materializes
///
/// One transaction input, spending the named outpoint from a previous
/// output carrying exactly `spent_asset`, `spent_value`, and
/// `spent_program`. The spent-output fields are stated in their
/// *transaction* encoding, prefix byte included, which is not always the
/// order the introspection primitives push: an explicit amount is stored
/// most significant byte first in the field and reaches the stack least
/// significant byte first. The executor writes the field; the expected
/// stack states the stack form.
///
/// # What only the executor can supply
///
/// Two fields are optional because no fixture can know them. An outpoint
/// names a real funding output the executor created, and the program of
/// the input under validation commits to the very script the fixture
/// carries, so it can only be computed once that script exists. `None`
/// means the executor supplies the value, and a fixture states no
/// expectation about it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureInput {
    /// The spent outpoint's transaction identifier, in the byte order
    /// the outpoint field carries it, where the fixture fixes one.
    pub outpoint_txid: Option<[u8; 32]>,
    /// The spent outpoint's index, where the fixture fixes one.
    pub outpoint_index: Option<u32>,
    /// The spent output's asset field, prefix included, where the
    /// fixture fixes one.
    ///
    /// `None` for a funding output the executor created: which asset a
    /// development network issues is the network's fact, not a fixture's,
    /// and a fixture that named one would be describing a chain rather
    /// than a target.
    pub spent_asset: Option<Vec<u8>>,
    /// The spent output's value field, prefix included, in transaction
    /// byte order, where the fixture fixes one.
    pub spent_value: Option<Vec<u8>>,
    /// The spent output's program, as complete script bytes, where the
    /// fixture fixes one.
    ///
    /// A witness program's script is its version opcode and its pushed
    /// payload; anything else is an ordinary script, which the program
    /// introspection primitives replace with a digest. The input under
    /// validation states `None`: its program commits to the fixture's own
    /// leaf script, so only the executor can compute it.
    pub spent_program: Option<Vec<u8>>,
    /// The input's sequence number.
    pub sequence: u32,
    /// The input's issuance fields, where it carries an issuance.
    pub issuance: Option<FixtureIssuance>,
    /// The input's initial witness stack, deepest item first.
    ///
    /// Only the input under validation needs a witness that satisfies a
    /// script; the executor may spend every other input however the
    /// materialization requires, since no fixture states an expectation
    /// about them.
    pub witness: Vec<Vec<u8>>,
}

/// The issuance fields of one input.
///
/// # What the executor materializes
///
/// The input's issuance record. A zero blinding nonce marks an issuance
/// and a nonzero one marks a reissuance, which is a target fact rather
/// than a convention this package chose.
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
///
/// # What the executor materializes
///
/// One transaction output carrying exactly these four fields as stated,
/// prefix bytes included. The asset and value axes are independent: a
/// fixture may state an explicit asset with a blinded value or the
/// reverse, and an executor that derives one from the other is not
/// materializing what it was handed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureOutput {
    /// The asset field, prefix included, where the fixture fixes one.
    ///
    /// `None` leaves the executor to pay in whatever asset its network
    /// issues, which is the only asset it can pay in.
    pub asset: Option<Vec<u8>>,
    /// The value field, prefix included, in transaction byte order.
    pub value: Vec<u8>,
    /// The nonce field, prefix included; empty for the absent form.
    pub nonce: Vec<u8>,
    /// The output program, as complete script bytes, where the fixture
    /// fixes one.
    pub program: Option<Vec<u8>>,
}

/// The script-path data the executor spends through.
///
/// # What the executor materializes
///
/// A script-path spend of the input under validation whose leaf carries
/// exactly `script` at exactly `leaf_version`. The control block is
/// computed by the executor from the leaf it built, so `control` is
/// `None` unless a fixture pins it — and no fixture states an
/// expectation about its bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureScriptPath {
    /// The leaf version byte.
    pub leaf_version: u8,
    /// The exact leaf script bytes.
    pub script: Vec<u8>,
    /// The control data, where the fixture pins it.
    pub control: Option<Vec<u8>>,
}

/// The generic transaction context an introspection primitive reads.
///
/// Input and output counts are not fields: they are the lengths of the
/// two vectors, and a separately stated count could disagree with them.
///
/// # What the executor materializes
///
/// One complete target transaction on the development network the run is
/// bound to, with exactly these inputs and outputs in exactly this order,
/// this version, and this locktime, validating the input at
/// `current_input_index` through `script_path`. Every value here is
/// public generic test material: no field names an attestation-contract
/// object, and none carries production secret material.
///
/// Fees, change, and any further outputs an executor's own machinery
/// would ordinarily add are *not* admitted — an added output would move
/// every output index a fixture states, and the fixture's expectations
/// would then describe a different transaction from the one that ran.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrimitiveExecutionContext {
    /// The transaction version.
    pub version: u32,
    /// The transaction locktime.
    pub locktime: u32,
    /// Which input is being validated.
    pub current_input_index: u32,
    /// The transaction's inputs, in transaction order.
    pub inputs: Vec<FixtureInput>,
    /// The transaction's outputs, in transaction order.
    pub outputs: Vec<FixtureOutput>,
    /// The script path being spent.
    pub script_path: FixtureScriptPath,
}

/// The script bytes one fixture hands the executor, and where they came
/// from.
#[derive(Clone, Debug)]
pub enum FixtureScript<'a> {
    /// One typed program, encoded through the reviewed contract.
    Typed(&'a TapscriptProgram),
    /// Bytes stated directly, because the form under test is one no
    /// typed program expresses.
    DeliberatelyMalformed(Vec<u8>),
}

/// Everything one fixture states, gathered for construction.
///
/// A parts struct rather than a long argument list: the fields are named
/// at every call site, so a census this long cannot silently transpose
/// two of them.
#[derive(Debug)]
pub struct FixtureStatement<'a> {
    /// Which case the fixture answers for.
    pub case: NativeCaseId,
    /// The script, and where its bytes came from.
    pub script: FixtureScript<'a>,
    /// The exact initial stack, deepest item first.
    pub initial_stack: &'a [StackItem],
    /// The transaction context, where the case needs one.
    pub context: Option<PrimitiveExecutionContext>,
    /// What the reviewed contract requires the target to do.
    pub expected: ExpectedPrimitiveOutcome,
    /// Whether the fixture is stated at the reviewed leaf version.
    pub leaf_version: LeafVersionStatus,
    /// The leaf version byte, where the case states an unreviewed one.
    pub unreviewed_leaf_version: Option<u8>,
    /// Which rule the stated verdict belongs to.
    pub enforcement_layer: EnforcementLayer,
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
    leaf_version_status: LeafVersionStatus,
    enforcement_layer: EnforcementLayer,
    script_source: FixtureScriptSource,
    script: Vec<u8>,
    initial_stack: Vec<Vec<u8>>,
    context: Option<PrimitiveExecutionContext>,
    expected: ExpectedPrimitiveOutcome,
    expected_resources: ExpectedResourceObservation,
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
        binding: &ReviewedDevelopmentBinding,
        case: NativeCaseId,
        program: &TapscriptProgram,
        initial_stack: &[StackItem],
        context: Option<PrimitiveExecutionContext>,
        expected: ExpectedPrimitiveOutcome,
    ) -> Result<Self, NativeConformanceError> {
        Self::state(
            target,
            binding,
            FixtureStatement {
                case,
                script: FixtureScript::Typed(program),
                initial_stack,
                context,
                expected,
                leaf_version: LeafVersionStatus::Reviewed,
                unreviewed_leaf_version: None,
                enforcement_layer: EnforcementLayer::Consensus,
            },
        )
    }

    /// States one fixture from its complete typed parts.
    ///
    /// # Errors
    ///
    /// [`NativeConformanceError::TargetContractMismatch`] when the
    /// reviewed contract's execution domain has no wire spelling, and
    /// when a statement's leaf version disagrees with its own status —
    /// an unreviewed case that names the reviewed byte would establish
    /// nothing, and a reviewed case cannot name any other byte.
    pub fn state(
        target: &ReviewedElementsTapscriptDefinition,
        binding: &ReviewedDevelopmentBinding,
        statement: FixtureStatement<'_>,
    ) -> Result<Self, NativeConformanceError> {
        let definition = target.definition();
        let domain = WireExecutionDomain::of(definition.execution_domain())
            .ok_or(NativeConformanceError::TargetContractMismatch)?;
        let reviewed_leaf = definition.leaf_version().get();
        let leaf_version = match (statement.leaf_version, statement.unreviewed_leaf_version) {
            (LeafVersionStatus::Reviewed, None) => reviewed_leaf,
            (LeafVersionStatus::Unreviewed, Some(byte)) if byte != reviewed_leaf => byte,
            _ => return Err(NativeConformanceError::TargetContractMismatch),
        };

        let (script_source, script) = match statement.script {
            // Exact bytes, produced once through the typed program.
            FixtureScript::Typed(program) => {
                (FixtureScriptSource::TypedProgram, program.encode(target))
            }
            FixtureScript::DeliberatelyMalformed(bytes) => {
                (FixtureScriptSource::DeliberatelyMalformed, bytes)
            }
        };
        let initial_stack: Vec<Vec<u8>> = statement
            .initial_stack
            .iter()
            .map(|item| item.bytes().to_vec())
            .collect();
        // The script path is filled in from the fixture rather than
        // accepted from the caller, so a context cannot claim a leaf
        // version or a leaf script other than the one that ran. Only
        // the control data, which no fixture states an expectation
        // about, is left as it was offered.
        let context = statement.context.map(|context| PrimitiveExecutionContext {
            script_path: FixtureScriptPath {
                leaf_version,
                script: script.clone(),
                control: context.script_path.control,
            },
            ..context
        });
        let expected_resources = ExpectedResourceObservation {
            script_bytes: ResourceExpectation::Exact(
                u64::try_from(script.len()).unwrap_or(u64::MAX),
            ),
            initial_stack_items: ResourceExpectation::Exact(
                u64::try_from(initial_stack.len()).unwrap_or(u64::MAX),
            ),
            peak_stack_items: ResourceExpectation::RecordedOnly,
            peak_altstack_items: ResourceExpectation::RecordedOnly,
            maximum_element_bytes: ResourceExpectation::RecordedOnly,
            validation_budget_used: ResourceExpectation::RecordedOnly,
            transaction_weight: ResourceExpectation::RecordedOnly,
        };

        // A malformed fixture does not acquire a report subject. The
        // canonical census is first-party source and is expected to be
        // well shaped, but a fixture is a public construction surface,
        // and "well shaped" is a property the constructor establishes
        // rather than a property callers are trusted to preserve.
        if let Some(context) = context.as_ref() {
            validate_context(statement.case, context)?;
        }

        Ok(Self {
            case: statement.case,
            target_contract_version: definition.version().get(),
            network_id: binding.binding().network_id(),
            genesis_id: binding.binding().genesis_id(),
            execution_domain: domain,
            leaf_version,
            leaf_version_status: statement.leaf_version,
            enforcement_layer: statement.enforcement_layer,
            script_source,
            script,
            initial_stack,
            context,
            expected: statement.expected,
            expected_resources,
        })
    }

    /// The complete comparison form of this fixture.
    ///
    /// Everything the executor was handed, plus the claims the case bears
    /// on. This is what a report row carries: a case ordinal navigates
    /// one census and identifies nothing across two.
    #[must_use]
    pub fn projection(&self) -> PrimitiveFixtureProjection {
        PrimitiveFixtureProjection {
            case: self.case,
            target_contract_version: self.target_contract_version,
            network_id: self.network_id,
            genesis_id: self.genesis_id,
            execution_domain: self.execution_domain,
            leaf_version: self.leaf_version,
            leaf_version_status: self.leaf_version_status,
            enforcement_layer: self.enforcement_layer,
            script_source: self.script_source,
            script: self.script.clone(),
            initial_stack: self.initial_stack.clone(),
            context: self.context.clone(),
            expected: self.expected.clone(),
            expected_resources: self.expected_resources,
            claims: claims_of(self),
        }
    }

    /// Exactly what the executor is handed, and nothing it is asked to
    /// agree with.
    ///
    /// # The subject without the answer
    ///
    /// This is the projection minus the expectation: the script, the
    /// initial stack, the transaction context, the enforcement layer, the
    /// leaf version, and the facts the case is stated against. The
    /// expected outcome, the expected resource figures, and the claim set
    /// are all absent, because they are what the harness compares the
    /// executor's answer *with* and an executor that could read them
    /// would be reading the answer
    /// `(´[PLAN-rule:guide11-exec:request-subject]´)`.
    ///
    /// It is one value with two uses, deliberately. It is the protocol
    /// revision-3 request payload, and it is what the transcript retains
    /// as the request that was actually sent — so the thing compared at
    /// evaluation is the thing that went over the wire, rather than a
    /// second description of it.
    #[must_use]
    pub fn subject(&self) -> PrimitiveExecutionSubject {
        PrimitiveExecutionSubject {
            case: self.case,
            target_contract_version: self.target_contract_version,
            network_id: self.network_id,
            genesis_id: self.genesis_id,
            execution_domain: self.execution_domain,
            leaf_version: self.leaf_version,
            leaf_version_status: self.leaf_version_status,
            enforcement_layer: self.enforcement_layer,
            script_source: self.script_source,
            script: self.script.clone(),
            initial_stack: self.initial_stack.clone(),
            context: self.context.clone(),
        }
    }

    /// Whether the fixture is stated at the reviewed leaf version.
    #[must_use]
    pub const fn leaf_version_status(&self) -> LeafVersionStatus {
        self.leaf_version_status
    }

    /// Which rule the fixture's verdict is stated at.
    #[must_use]
    pub const fn enforcement_layer(&self) -> EnforcementLayer {
        self.enforcement_layer
    }

    /// Where the fixture's exact script bytes came from.
    #[must_use]
    pub const fn script_source(&self) -> FixtureScriptSource {
        self.script_source
    }

    /// What the execution is expected to cost.
    #[must_use]
    pub const fn expected_resources(&self) -> ExpectedResourceObservation {
        self.expected_resources
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

/// Exactly what one executor was handed for one case.
///
/// # Why this is a type and not a filtered view
///
/// The execution subject and the expectation used to travel as one value,
/// because a fixture is one value and splitting it risked the harness and
/// the executor holding two different notions of what ran. Protocol
/// revision 3 splits them anyway, and this is the half that crosses the
/// boundary: everything the executor needs to perform the execution, and
/// no statement at all about what the result should be
/// `(´[PLAN-rule:guide11-exec:request-subject]´)`.
///
/// The other half never leaves the harness. That is the point: an
/// executor cannot discard an expectation it was never sent, so the
/// discipline is a property of the protocol rather than of an adapter's
/// good behaviour.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrimitiveExecutionSubject {
    /// Which case.
    pub case: NativeCaseId,
    /// The contract revision the case is stated against.
    pub target_contract_version: u32,
    /// The network the case is stated against.
    pub network_id: [u8; 32],
    /// The genesis identifier the case is stated against.
    pub genesis_id: [u8; 32],
    /// The execution domain.
    pub execution_domain: WireExecutionDomain,
    /// The leaf version byte.
    pub leaf_version: u8,
    /// Whether that byte is the reviewed one.
    pub leaf_version_status: LeafVersionStatus,
    /// Which rule the case is to be answered at.
    pub enforcement_layer: EnforcementLayer,
    /// Where the exact script bytes came from.
    pub script_source: FixtureScriptSource,
    /// The exact script bytes.
    pub script: Vec<u8>,
    /// The exact initial stack.
    pub initial_stack: Vec<Vec<u8>>,
    /// The transaction context, where the case needs one.
    pub context: Option<PrimitiveExecutionContext>,
}

/// The complete comparison form of one fixture.
///
/// Carried by every report row, so that a report states what was executed
/// rather than pointing at a census that may since have changed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrimitiveFixtureProjection {
    /// Which case.
    pub case: NativeCaseId,
    /// The contract revision the fixture was stated against.
    pub target_contract_version: u32,
    /// The network the fixture was stated against.
    pub network_id: [u8; 32],
    /// The genesis identifier the fixture was stated against.
    pub genesis_id: [u8; 32],
    /// The execution domain.
    pub execution_domain: WireExecutionDomain,
    /// The leaf version byte.
    pub leaf_version: u8,
    /// Whether that byte is the reviewed one.
    pub leaf_version_status: LeafVersionStatus,
    /// Which rule the stated verdict belongs to.
    pub enforcement_layer: EnforcementLayer,
    /// Where the exact script bytes came from.
    pub script_source: FixtureScriptSource,
    /// The exact script bytes.
    pub script: Vec<u8>,
    /// The exact initial stack.
    pub initial_stack: Vec<Vec<u8>>,
    /// The transaction context, where the case needs one.
    pub context: Option<PrimitiveExecutionContext>,
    /// What the reviewed contract requires the target to do.
    pub expected: ExpectedPrimitiveOutcome,
    /// What the execution is expected to cost.
    pub expected_resources: ExpectedResourceObservation,
    /// The typed claims the case bears on.
    pub claims: BTreeSet<NativeEvidenceClaim>,
}

/// Whether one transaction context is internally well shaped.
///
/// Local structure only: the relationships a fixture can state wrongly
/// without any executor being involved. It does not establish that a node
/// can materialize the transaction, which no static check can.
fn validate_context(
    case: NativeCaseId,
    context: &PrimitiveExecutionContext,
) -> Result<(), NativeConformanceError> {
    let malformed = || NativeConformanceError::MalformedFixtureContext(case);

    if context.inputs.is_empty() || context.outputs.is_empty() {
        return Err(malformed());
    }
    // The validated input must exist. An index past the end names an
    // input the executor would have to invent, and a fixture's
    // expectations would then describe a different transaction.
    let index = usize::try_from(context.current_input_index).map_err(|_| malformed())?;
    if index >= context.inputs.len() {
        return Err(malformed());
    }
    // The leaf script under validation is the fixture's own, welded in
    // above. A context whose path states no script would leave the
    // executor to choose one.
    if context.script_path.script.is_empty() {
        return Err(malformed());
    }

    for input in &context.inputs {
        // A stated field is a field the executor must write exactly. An
        // empty one is not a field, and a fixture leaving it to the
        // executor says so with `None` rather than with emptiness.
        for field in [input.spent_asset.as_deref(), input.spent_value.as_deref()]
            .into_iter()
            .flatten()
        {
            if field.is_empty() {
                return Err(malformed());
            }
        }
        if input.spent_program.as_deref().is_some_and(<[u8]>::is_empty) {
            return Err(malformed());
        }
        if let Some(issuance) = input.issuance.as_ref()
            && (issuance.asset_amount.is_empty() || issuance.inflation_keys_amount.is_empty())
        {
            return Err(malformed());
        }
    }
    for output in &context.outputs {
        if output.value.is_empty() || output.asset.as_deref().is_some_and(<[u8]>::is_empty) {
            return Err(malformed());
        }
        if output.program.as_deref().is_some_and(<[u8]>::is_empty) {
            return Err(malformed());
        }
    }

    Ok(())
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

/// The canonical primitive census, in the one trust state the native
/// evidence path accepts.
///
/// # Why the wrapper exists
///
/// [`PrimitiveFixtureSet`] is a public construction surface: any caller
/// may assemble any fixtures into one. That is useful for experiments and
/// necessary for the harness's own tests, and it is exactly what must not
/// be an evidence subject. A caller who may choose the census may file a
/// program that pushes a true literal under a case whose group names the
/// signature dimension, and every claim below reads the label rather than
/// the program — so the report states signature evidence for a run in
/// which no signature primitive executed
///.
///
/// The refusal is therefore about *provenance*, not about size or
/// completeness: a caller-assembled census large enough to fill every
/// required row would still be a census the caller chose. Only
/// [`canonical_fixture_set`] constructs this wrapper, so the evidence path
/// asks its question about the repository's own evidence plan and about
/// nothing else.
///
/// # What the wrapper is not
///
/// It is not a claim that the census is complete, correct, or sufficient.
/// It is the single fact that this value came from the canonical
/// generator rather than from a caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalPrimitiveFixtureSet {
    fixtures: PrimitiveFixtureSet,
}

impl CanonicalPrimitiveFixtureSet {
    /// The census itself.
    #[must_use]
    pub const fn fixtures(&self) -> &PrimitiveFixtureSet {
        &self.fixtures
    }

    /// Consumes the canonical state, yielding the bare census.
    #[must_use]
    pub fn into_fixtures(self) -> PrimitiveFixtureSet {
        self.fixtures
    }

    /// The fixtures, in canonical case order.
    pub fn iter(&self) -> impl Iterator<Item = &PrimitiveFixture> {
        self.fixtures.iter()
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
        self.fixtures.contains(case)
    }

    /// An arbitrary census wrapped as though it were canonical, for the
    /// crate's own tests.
    ///
    /// Not public, and deliberately the only way to reach the state
    /// without the generator. It exists so the regressions can exercise
    /// the *second* line of defence — the regeneration comparison in
    /// [`crate::validate::evaluate`] — rather than only the type. Outside
    /// this crate the type alone is the boundary, which is why there is
    /// no public equivalent.
    #[cfg(test)]
    pub(crate) const fn wrap_for_tests(fixtures: PrimitiveFixtureSet) -> Self {
        Self { fixtures }
    }
}

impl<'a> IntoIterator for &'a CanonicalPrimitiveFixtureSet {
    type Item = &'a PrimitiveFixture;
    type IntoIter = std::collections::btree_map::Values<'a, NativeCaseId, PrimitiveFixture>;

    fn into_iter(self) -> Self::IntoIter {
        self.fixtures.into_iter()
    }
}

/// The canonical fixture census of this repository.
///
/// The complete primitive matrix, authored against
/// the reviewed contract and independent published vectors by the
/// crate-internal census module. Its content, its coverage, and what it
/// deliberately does not cover are documented there.
///
/// This is the only constructor of [`CanonicalPrimitiveFixtureSet`], and
/// therefore the only route to a gate-eligible primitive subject.
///
/// # Errors
///
/// [`NativeConformanceError::DuplicateFixtureCase`] when two fixtures
/// declare one case identity, and
/// [`NativeConformanceError::TargetContractMismatch`] when a fixture
/// cannot be stated against the reviewed contract at all.
pub fn canonical_fixture_set(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
) -> Result<CanonicalPrimitiveFixtureSet, NativeConformanceError> {
    Ok(CanonicalPrimitiveFixtureSet {
        fixtures: crate::census::canonical_census(target, binding)?,
    })
}
