//! The secretless target-native conformance harness.
//!
//! # Boundary
//!
//! This package owns the *mechanics* of asking an external Elements
//! executor what it did: the fixture language, the child-process
//! protocol, the executor driver, the typed report, and the first-party
//! evidence plan. It owns no target semantics whatever. Every reviewed
//! fact a fixture is stated against — the execution domain, the leaf
//! version, the primitive contracts, the encodings, the evidence
//! requirements — belongs to `target-elements`, and the exact script
//! bytes a fixture carries are produced by a typed `tapscript` program
//! rather than assembled here.
//!
//! # What the harness cannot establish
//!
//! An executor is caller-selected code. Selecting one grants it
//! execution authority (ADR-015), and the harness does not authenticate
//! it, sandbox it, or prove it independent of anything. Its handshake is
//! *provenance*: a dishonest executor can misdescribe itself, and the
//! report never claims otherwise. A mock executor is admitted for
//! protocol and failure-path tests and can never satisfy the
//! target-native gate — the gate requires an explicit nonmock selection,
//! recorded as ordinary provenance.
//!
//! # No secret material
//!
//! No interface here accepts an RPC username, a password, a bearer
//! token, a cookie path, a private key, a signing nonce, a production
//! blinding factor, a private opening, a wallet path, or a production
//! endpoint, and none may be added. If an executor needs to authenticate
//! to a node, it establishes that boundary itself, outside this process.
//!
//! # No identity
//!
//! Nothing here is hashed. There is no report digest, no fixture-set
//! digest, no executor digest, and no field reserved for one: a report
//! is compared by its typed content and its exact bytes, and no
//! persistent report identity is admitted until a cross-process release
//! consumer exists (Guide-9 §1.8, ADR-016).
//!
//! # What a native run can and cannot observe
//!
//! A validating node answers one question: was this spend valid, and
//! coarsely, why not. It exposes no interpreter stack, and it reports one
//! reason for several reviewed causes. Every expectation here is shaped
//! by that: a verdict, the set of failure classes the contract admits,
//! and the exact stacks as *static* statements, compared only when an
//! executor happens to report one.
//!
//! The reviewed domain also requires evaluation to finish with exactly
//! one true item, and the reviewed primitive census has no equality,
//! drop, or verify primitive to reduce a deeper stack with. So several
//! primitives have no reachable accepting case at all, and what their
//! cases establish is the number of items the primitive pushed. The
//! census says which primitives those are and why.
//!
//! # State
//!
//! Implemented: the package boundary, the typed error root, the wire
//! vocabulary that names reviewed target identities, the generic fixture
//! language, the secretless executor protocol, the external executor
//! driver, the typed conformance report, the Guide-9 evidence plan, the
//! gate, and the canonical primitive census.
//!
//! Implemented alongside it, and kept apart from it throughout: the
//! compound-prototype fixture language, the constructor and wide-floor
//! case matrices, the typed prototype report, and the prototype
//! validator and gate. The two are separate types answering separate
//! questions, and one report never carries both — a primitive census
//! establishes what one reviewed primitive did, and a prototype matrix
//! establishes whether a multi-step construction held together across a
//! whole target output.
//!
//! Not covered by the census, and recorded as residuals rather than
//! filled in: a signature over a transaction sighash, blinded fields,
//! issuing inputs, an absent introspection context, and any execution
//! domain other than the reviewed one.
//!
//! # Public modules
//!
//! The primitive lane, in the order a run uses them:
//!
//! - [`fixture`] — the generic fixture language: [`fixture::NativeCaseId`],
//!   [`fixture::PrimitiveFixture`], [`fixture::PrimitiveFixtureSet`], and
//!   [`fixture::canonical_fixture_set`], which builds the whole
//!   reviewed-primitive census.
//! - [`protocol`] — the secretless wire vocabulary: the schema constant,
//!   the handshake, the executor capability set, the verdict, the 34
//!   observed failure classes, and the response-shape rules.
//! - [`executor`] — the driver: [`executor::ExecutorConfiguration`],
//!   [`executor::ExecutorTrust`], [`executor::execute`], and the
//!   [`executor::ExecutionTranscript`] it returns.
//! - [`claim`] — the typed claim census beneath the broad evidence
//!   requirements, and which claims each fixture bears on.
//! - [`validate`] — [`validate::guide_nine_evidence_plan`],
//!   [`validate::evaluate`], [`validate::validate_native_report`], and
//!   [`validate::gate`].
//! - [`report`] — the typed report the lane produces.
//!
//! The prototype lane, kept apart from the primitive lane throughout:
//!
//! - [`prototype`] — the compound fixture language, the claim
//!   vocabulary, and the two case matrices.
//! - [`prototype_program`] — the typed prototype programs and their
//!   measured resource projections.
//! - [`prototype_validate`] — the prototype evaluator, report validator,
//!   and gate.
//! - [`prototype_report`] — the typed prototype report.
//!
//! The independent oracles the matrices are checked against:
//!
//! - [`constructor`] — public taproot and metadata arithmetic. No
//!   secret scalar appears anywhere in it.
//! - [`wide_floor`] — the wide-arithmetic floor domain and its oracle.
//!
//! - [`vocabulary`] — the explicit wire spellings for reviewed target
//!   identities that cross the process boundary.
//! - [`error`] — [`NativeConformanceError`], the crate's single error
//!   root, re-exported at the crate root.
//!
//! # The workflow
//!
//! ```text
//! reviewed_elements_tapscript()                 the reviewed static contract
//! validate_reviewed_development_binding(..)     the development binding
//! canonical_fixture_set(&target, &binding)      or hand-built fixtures
//! ExecutorConfiguration::new(path, trust, ..)   the caller selects the executor
//! executor::execute(..)          -> ExecutionTranscript
//! validate::evaluate(..)         -> NativeConformanceReport
//! validate::validate_native_report(..) -> ValidatedNativeConformanceReport
//! validate::gate(&validated)     -> Ok(()) only for a nonmock executor
//! ```
//!
//! Two types have no public constructor, deliberately.
//! [`executor::ExecutionTranscript`] can only be obtained by actually
//! running an executor, so a caller cannot fabricate one; and
//! [`validate::ValidatedNativeConformanceReport`] can only be obtained
//! from [`validate::validate_native_report`], which recomputes every
//! field rather than trusting the report it was handed.
//!
//! A worked example of the whole chain, the full public-API tour, and
//! the executor-authority rules are in the package README.
//!
//! # Errors
//!
//! Every fallible operation returns [`NativeConformanceError`], which
//! has 68 variants declared in `src/error.rs` and is
//! `#[non_exhaustive]`. Every variant is a branch that runs: there is
//! no catch-all, and no variant carries child-process detail that could
//! leak an executor's environment.
//!
//! The one to know by name is
//! `NativeConformanceError::MockExecutorCannotSatisfyNativeGate`. It is
//! the first thing [`validate::gate`] and
//! [`prototype_validate::prototype_gate`] check, before any other
//! condition, and nothing else in the crate enforces it: a mock runs,
//! reports, and validates like any other executor, and is refused only
//! at the gate.

#![forbid(unsafe_code)]

mod census;
pub mod claim;
pub mod constructor;
pub mod error;
pub mod executor;
pub mod fixture;
pub mod protocol;
pub mod prototype;
pub mod prototype_program;
pub mod prototype_report;
pub mod prototype_validate;
pub mod report;
pub mod validate;
pub mod vocabulary;
pub mod wide_floor;

pub use error::NativeConformanceError;

#[cfg(test)]
mod tests;
