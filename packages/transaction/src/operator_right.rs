//! Affine operator authority within one caller-maintained registry.
//!
//! At-most-once ledger commit is the only consensus guarantee. This registry
//! limits accepted signing actions within its lifetime; it is neither durable
//! storage nor a global signer lock. Callers must retain it across retries.
//! The conceptual register's equivocation candidate remains a candidate:
//! implementing these evidence obligations does not adopt it.

use std::collections::BTreeMap;
use std::sync::Arc;

use linker::CandidateDeploymentIdentity;
use tapscript::StateConstructorGeneration;
use target_elements::TargetContractVersion;

use crate::bytes::{AssetId, InputWitness, Outpoint};
use crate::operator_signing::{
    OperatorAuthorizedCandidate, OperatorEvidenceStanding, OperatorSigningRefusal,
    OperatorSigningRequest,
};
use crate::taproot::{Digest32, tagged_hash};

/// Minimal caller-evidenced context pending the later branch-indexed model.
///
/// The constructor checks identity shape, not chain evidence. Callers must
/// establish the branch and checkpoint externally before offering a new context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BranchContext {
    identifier: Digest32,
    checkpoint: u64,
}

impl BranchContext {
    /// Checks a branch identifier and checkpoint ordinal.
    ///
    /// # Errors
    /// Refuses an all-zero branch identifier.
    pub fn new(identifier: Digest32, checkpoint: u64) -> Result<Self, RightRefusal> {
        if identifier == [0; 32] {
            return Err(RightRefusal::ZeroBranchIdentifier { checkpoint });
        }
        Ok(Self {
            identifier,
            checkpoint,
        })
    }

    /// The caller's branch identifier.
    #[must_use]
    pub const fn identifier(&self) -> &Digest32 {
        &self.identifier
    }
    /// The caller's evidenced checkpoint ordinal.
    #[must_use]
    pub const fn checkpoint(&self) -> u64 {
        self.checkpoint
    }
}

/// The asset identity or issuance input from which a thread starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateThreadProvenance {
    /// An existing asset, without a claim about its earlier history.
    ExistingAsset(AssetId),
    /// The outpoint whose spend carries issuance and supplies its entropy input.
    Issuance(Outpoint),
}

/// Which branches may carry continuations of an anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateCheckpointPolicy {
    /// Only the anchor's exact branch identifier, at any checkpoint ordinal.
    ExactBranch,
    /// Any branch at or beyond this checkpoint ordinal, including reorganizations.
    AtOrBeyond(u64),
}

/// The accepted continuity evidence still owed by a thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum StateContinuityEvidence {
    /// Accepted continuity and history evidence have not yet been attached.
    Outstanding,
}

/// Whether the starting point is a fixture or an observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateThreadOrigin {
    /// A fixture: not genesis, trusted setup, earlier history, or production STATE.
    Synthetic,
    /// An observed starting point, with continuity evidence still outstanding.
    Observed,
}

/// Caller-supplied target acceptance of a continuation transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateContinuationStanding {
    /// The caller states that the target accepted this continuation transaction.
    Accepted,
    /// No target acceptance is asserted.
    NotAccepted,
}

/// One offered continuation and its caller-supplied acceptance fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateContinuation {
    /// The branch and checkpoint where the continuation is offered.
    pub branch: BranchContext,
    /// The successor output of the continuation transaction.
    pub successor: Outpoint,
    /// Acceptance is a supplied fact, not evidence verified by this record.
    pub standing: StateContinuationStanding,
}

/// A shape or conditional-uniqueness refusal for a state thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateThreadRefusal {
    /// The anchor's branch identifier was all zero.
    ZeroBranchIdentifier {
        /// The offered checkpoint ordinal.
        checkpoint: u64,
    },
    /// An offered continuation's branch or ordinal was outside the policy.
    OutsideCheckpointPolicy(BranchContext),
    /// Two accepted continuations named the same branch identifier.
    Equivocation(Digest32),
}

/// Typed provenance for one caller-anchored predecessor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateThreadAnchor {
    deployment: CandidateDeploymentIdentity,
    branch: BranchContext,
    starting_outpoint: Outpoint,
    provenance: StateThreadProvenance,
    generation: StateConstructorGeneration,
    checkpoint_policy: StateCheckpointPolicy,
    continuity_evidence: StateContinuityEvidence,
    origin: StateThreadOrigin,
}

impl StateThreadAnchor {
    /// Checks anchor shape without establishing provenance or continuity.
    ///
    /// `branch` supplies the identifier and checkpoint ordinal checked by
    /// [`BranchContext::new`]. The typed [`Outpoint`] already excludes reserved
    /// index bits, including the maximum integer index. Both origins are
    /// admissible with outstanding evidence; deployment identity is not a
    /// proof that this thread began at genesis.
    ///
    /// # Errors
    /// Refuses a zero branch identifier.
    pub fn new(
        deployment: CandidateDeploymentIdentity,
        branch: (Digest32, u64),
        starting_outpoint: Outpoint,
        provenance: StateThreadProvenance,
        generation: StateConstructorGeneration,
        checkpoint_policy: StateCheckpointPolicy,
        origin: StateThreadOrigin,
    ) -> Result<Self, StateThreadRefusal> {
        let branch = BranchContext::new(branch.0, branch.1).map_err(|_| {
            StateThreadRefusal::ZeroBranchIdentifier {
                checkpoint: branch.1,
            }
        })?;
        Ok(Self {
            deployment,
            branch,
            starting_outpoint,
            provenance,
            generation,
            checkpoint_policy,
            continuity_evidence: StateContinuityEvidence::Outstanding,
            origin,
        })
    }

    /// The named network and genesis identity.
    #[must_use]
    pub const fn deployment(&self) -> &CandidateDeploymentIdentity {
        &self.deployment
    }
    /// The checked starting branch context.
    #[must_use]
    pub const fn branch(&self) -> &BranchContext {
        &self.branch
    }
    /// The predecessor where this thread starts.
    #[must_use]
    pub const fn starting_outpoint(&self) -> Outpoint {
        self.starting_outpoint
    }
    /// The asset or issuance provenance supplied by the caller.
    #[must_use]
    pub const fn provenance(&self) -> StateThreadProvenance {
        self.provenance
    }
    /// The constructor recipe's typed generation.
    #[must_use]
    pub const fn generation(&self) -> StateConstructorGeneration {
        self.generation
    }
    /// The policy selecting eligible branches.
    #[must_use]
    pub const fn checkpoint_policy(&self) -> StateCheckpointPolicy {
        self.checkpoint_policy
    }
    /// The standing of accepted continuity evidence.
    #[must_use]
    pub const fn continuity_evidence(&self) -> StateContinuityEvidence {
        self.continuity_evidence
    }
    /// The starting point's synthetic or observed origin.
    #[must_use]
    pub const fn origin(&self) -> StateThreadOrigin {
        self.origin
    }

    /// Checks conditional at-most-one accepted continuation per selected branch.
    ///
    /// Uniqueness of the anchor is a hypothesis supplied by the caller's
    /// provenance, never proved here. The caller also supplies target acceptance
    /// and the fact that each candidate continues this anchored predecessor.
    /// Under [`StateCheckpointPolicy::ExactBranch`], the selected branch is the
    /// anchor's identifier, regardless of checkpoint ordinal. Under
    /// [`StateCheckpointPolicy::AtOrBeyond`], selected branches are the offered
    /// identifiers whose checkpoint ordinals meet the floor. Branch identity,
    /// not checkpoint ordinal, groups continuations: a second accepted entry
    /// for the same identifier refuses, even with the same successor.
    /// Success returns at most one accepted continuation for each selected
    /// branch, sorted by identifier; unaccepted entries are omitted.
    /// For a synthetic origin, a positive result makes no genesis claim and
    /// proves no global origin uniqueness; see [`StateThreadContinuations::RESIDUAL`].
    ///
    /// # Errors
    /// Refuses every out-of-policy candidate, including unaccepted ones, or
    /// two accepted continuations sharing a branch identifier.
    pub fn check_continuations(
        &self,
        candidates: &[StateContinuation],
    ) -> Result<StateThreadContinuations, StateThreadRefusal> {
        let mut accepted = BTreeMap::new();
        for candidate in candidates {
            let selected = match self.checkpoint_policy {
                StateCheckpointPolicy::ExactBranch => {
                    candidate.branch.identifier() == self.branch.identifier()
                }
                StateCheckpointPolicy::AtOrBeyond(floor) => candidate.branch.checkpoint() >= floor,
            };
            if !selected {
                return Err(StateThreadRefusal::OutsideCheckpointPolicy(
                    candidate.branch,
                ));
            }
            if candidate.standing == StateContinuationStanding::Accepted
                && accepted
                    .insert(*candidate.branch.identifier(), *candidate)
                    .is_some()
            {
                return Err(StateThreadRefusal::Equivocation(
                    *candidate.branch.identifier(),
                ));
            }
        }
        Ok(StateThreadContinuations {
            accepted: accepted.into_values().collect(),
            origin: self.origin,
        })
    }
}

/// Accepted continuations under the caller's unique-anchor hypothesis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateThreadContinuations {
    accepted: Vec<StateContinuation>,
    origin: StateThreadOrigin,
}

impl StateThreadContinuations {
    /// A positive check supplies neither genesis nor global origin uniqueness.
    pub const RESIDUAL: &'static str = "conditional on a uniquely anchored predecessor; no genesis claim or global origin uniqueness; synthetic origin is not protocol genesis, trusted setup, earlier root history, or production STATE and authorizes nothing of value";

    /// At most one accepted continuation per selected branch identifier.
    #[must_use]
    pub fn accepted(&self) -> &[StateContinuation] {
        &self.accepted
    }
    /// The anchor's origin, preserved without promotion by a positive check.
    #[must_use]
    pub const fn origin(&self) -> StateThreadOrigin {
        self.origin
    }
}

/// The complete version, deployment, and operator identity of one right.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RightScope {
    deployment: CandidateDeploymentIdentity,
    branch: BranchContext,
    predecessor: Outpoint,
    operator: Vec<u8>,
    revision: TargetContractVersion,
}

impl RightScope {
    /// Derives the scope from a checked frozen request.
    ///
    /// The established operator profile has one source selection per revision.
    /// Its committed revision therefore identifies the selected profile here.
    ///
    /// # Errors
    /// Refuses a request without its selected predecessor input.
    pub fn new(
        request: &OperatorSigningRequest<'_>,
        branch: BranchContext,
    ) -> Result<Self, RightRefusal> {
        Ok(Self {
            deployment: request.binding().deployment().clone(),
            branch,
            predecessor: request.predecessor_outpoint().ok_or_else(|| {
                RightRefusal::PredecessorAbsent {
                    input_index: request.input_index(),
                }
            })?,
            operator: request.binding().key().bytes().to_vec(),
            revision: request.binding().capability_revision(),
        })
    }

    /// The binding's network and genesis identity.
    #[must_use]
    pub const fn deployment(&self) -> &CandidateDeploymentIdentity {
        &self.deployment
    }
    /// The caller's branch and checkpoint.
    #[must_use]
    pub const fn branch(&self) -> &BranchContext {
        &self.branch
    }
    /// The spent version selected by the frozen input.
    #[must_use]
    pub const fn predecessor(&self) -> Outpoint {
        self.predecessor
    }
    /// The selected operator's public key bytes.
    #[must_use]
    pub fn operator(&self) -> &[u8] {
        &self.operator
    }
    /// The selected profile's capability revision.
    #[must_use]
    pub const fn capability_revision(&self) -> TargetContractVersion {
        self.revision
    }
}

/// An affine token issued by exactly one registry.
///
/// Dropping it abandons access without reopening issuance. Failed operations
/// return the token in [`RightFailure`]; successful signing never returns it.
///
/// ```compile_fail,E0599
/// use transaction::ConstructionRight;
/// fn duplicate(right: ConstructionRight) { let _second = right.clone(); }
/// ```
/// ```compile_fail,E0451
/// use transaction::{ConstructionRight, RightScope};
/// fn forge(scope: RightScope) {
///     let _right = ConstructionRight { scope, ordinal: 0, registry: Default::default() };
/// }
/// ```
#[derive(Debug)]
pub struct ConstructionRight {
    scope: RightScope,
    ordinal: usize,
    registry: Arc<()>,
}

impl ConstructionRight {
    /// The immutable scope this token carries.
    #[must_use]
    pub const fn scope(&self) -> &RightScope {
        &self.scope
    }
    /// The issuance position within the registry.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
}

/// Diagnostics identifying an issued right.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RightIdentity {
    /// Every component of the affected scope.
    pub scope: Box<RightScope>,
    /// The original issuance position.
    pub ordinal: usize,
}

/// A named registry refusal with its affected values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RightRefusal {
    /// The branch identifier was all zero.
    ZeroBranchIdentifier {
        /// The offered checkpoint.
        checkpoint: u64,
    },
    /// The frozen input had no predecessor.
    PredecessorAbsent {
        /// The selected input.
        input_index: u32,
    },
    /// Issuance would duplicate an outstanding capability.
    Outstanding(RightIdentity),
    /// Issuance or consumption would reacquire an already consumed right.
    Consumed(RightIdentity),
    /// Retry offered a competing candidate.
    CompetingCandidate {
        /// The consumed right.
        right: RightIdentity,
        /// The bound bytes' digest.
        bound: Digest32,
        /// The offered bytes' digest.
        offered: Digest32,
    },
    /// Replacement followed successful signing.
    ReplacementAfterSigning(RightIdentity),
    /// An indeterminate right cannot be reopened.
    Indeterminate(RightIdentity),
    /// Reorganization invalidated this exact branch context.
    StaleBranch(BranchContext),
    /// This registry never issued the scope.
    NoRight(Box<RightScope>),
    /// Consumption offered bytes other than those bound at issuance or replacement.
    MismatchedBytes {
        /// The outstanding right.
        right: RightIdentity,
        /// The bound bytes' digest.
        bound: Digest32,
        /// The offered bytes' digest.
        offered: Digest32,
    },
    /// The token belongs to a different registry or issuance.
    ForeignRight(RightIdentity),
    /// The request or returned artifact names another scope.
    ScopeMismatch {
        /// The token's scope.
        bound: Box<RightScope>,
        /// The request's scope.
        offered: Box<RightScope>,
    },
    /// Dispatch or retry requires successful signing.
    NotSigned(RightIdentity),
    /// Timeout requires dispatch first.
    NotDispatched(RightIdentity),
    /// The signing action returned no authorized artifact.
    Signing(Box<OperatorSigningRefusal>),
    /// The callback returned an artifact outside the checked request.
    UnexpectedAuthorization(RightIdentity),
}

/// A refused operation returns its token without granting new authority.
#[derive(Debug)]
pub struct RightFailure {
    /// The operation's named refusal.
    pub refusal: RightRefusal,
    /// The original token; terminal registry states still refuse it.
    pub right: ConstructionRight,
}

/// Owned evidence of the one authorization this registry accepted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CachedOperatorAuthorization {
    /// The issued scope and ordinal.
    pub right: RightIdentity,
    /// The exact candidate bytes' content digest.
    pub digest: Digest32,
    /// The witness actually returned by the signing boundary.
    pub witness: InputWitness,
    /// The verifier's in-process description, without native standing.
    pub standing: OperatorEvidenceStanding,
}

/// A live first authorization or owned evidence returned by retry.
#[derive(Debug, PartialEq, Eq)]
pub enum OperatorRightOutcome<'binding> {
    /// The first accepted signing action.
    Fresh(Box<OperatorAuthorizedCandidate<'binding>>),
    /// An identical retry, with no signing action.
    Cached(CachedOperatorAuthorization),
}

/// An action observed by this registry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NonEquivocationEvent {
    /// Exact bytes were bound and a token issued.
    Issued,
    /// Unsigned candidate bytes changed.
    Replaced,
    /// An authorized artifact was accepted and cached.
    Signed,
    /// An identical retry read the cached artifact.
    Cached,
    /// The signed artifact was marked dispatched.
    Dispatched,
    /// Dispatch timed out without a determinate result.
    TimedOut,
    /// Reorganization invalidated the scope and discarded its cache.
    Invalidated,
    /// A signing refusal left the right outstanding.
    SigningRefused,
    /// A callback returned another artifact, making the right indeterminate.
    UnexpectedAuthorization,
}

/// Signature-free metadata for one recorded action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NonEquivocationEntry {
    /// The affected scope and original issuance ordinal.
    pub right: RightIdentity,
    /// The action this registry observed.
    pub event: NonEquivocationEvent,
    /// The exact candidate's byte count.
    pub bytes_len: usize,
    /// A domain-separated content digest of those bytes.
    pub digest: Digest32,
}

/// An append-only record bounded by this registry's observations.
///
/// It records what this registry issued, signed, cached, replaced, dispatched,
/// timed out and invalidated. It claims nothing about signatures produced
/// outside the registry or about global signer honesty. Entries contain no
/// signatures; the separate cache holds only witnesses the registry saw.
/// Each state change appends one entry, as do failed signing and cached reads.
#[derive(Debug, Default)]
pub struct NonEquivocationRecord {
    entries: Vec<NonEquivocationEntry>,
}

impl NonEquivocationRecord {
    /// The immutable chronological observation sequence.
    #[must_use]
    pub fn entries(&self) -> &[NonEquivocationEntry] {
        &self.entries
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Outstanding,
    Signed,
    Dispatched,
    Indeterminate,
    Invalidated,
}

#[derive(Debug)]
struct Entry {
    identity: RightIdentity,
    bytes: Vec<u8>,
    state: State,
    cache: Option<CachedOperatorAuthorization>,
}

/// In-memory authority retained across one scope's construction and signing.
#[derive(Debug, Default)]
pub struct OperatorRightRegistry {
    identity: Arc<()>,
    entries: Vec<Entry>,
    stale: Vec<BranchContext>,
    record: NonEquivocationRecord,
}

impl OperatorRightRegistry {
    /// Issues one token and binds its exact unsigned candidate bytes.
    ///
    /// # Errors
    /// Refuses stale contexts and outstanding, consumed or indeterminate scopes.
    pub fn issue(
        &mut self,
        scope: RightScope,
        bytes: &[u8],
    ) -> Result<ConstructionRight, RightRefusal> {
        self.check_branch(scope.branch())?;
        if let Some(entry) = self
            .entries
            .iter()
            .find(|entry| *entry.identity.scope == scope)
        {
            return Err(match entry.state {
                State::Outstanding => RightRefusal::Outstanding(entry.identity.clone()),
                State::Indeterminate => RightRefusal::Indeterminate(entry.identity.clone()),
                _ => RightRefusal::Consumed(entry.identity.clone()),
            });
        }
        let ordinal = self.entries.len();
        self.entries.push(Entry {
            identity: RightIdentity {
                scope: Box::new(scope.clone()),
                ordinal,
            },
            bytes: bytes.to_vec(),
            state: State::Outstanding,
            cache: None,
        });
        self.append(ordinal, NonEquivocationEvent::Issued);
        Ok(ConstructionRight {
            scope,
            ordinal,
            registry: Arc::clone(&self.identity),
        })
    }

    /// Replaces only the bytes of an outstanding unsigned right.
    ///
    /// # Errors
    /// Returns the token with a stale, unknown, terminal or foreign-right refusal.
    pub fn replace(
        &mut self,
        right: ConstructionRight,
        bytes: &[u8],
    ) -> Result<ConstructionRight, Box<RightFailure>> {
        match self.replace_bytes(&right, bytes) {
            Ok(()) => Ok(right),
            Err(refusal) => Err(Box::new(RightFailure { refusal, right })),
        }
    }

    fn replace_bytes(
        &mut self,
        right: &ConstructionRight,
        bytes: &[u8],
    ) -> Result<(), RightRefusal> {
        let index = self.outstanding(right, true)?;
        if self.entries[index].bytes != bytes {
            self.entries[index].bytes = bytes.to_vec();
            self.append(index, NonEquivocationEvent::Replaced);
        }
        Ok(())
    }

    /// Executes one signing attempt after checking the token, scope and bytes.
    ///
    /// A refusal returns the token and leaves it outstanding. A successful
    /// callback consumes it. A callback returning another candidate makes the
    /// right indeterminate, since a signature was observed and cannot be undone.
    ///
    /// # Errors
    /// Returns the token with a registry or signing refusal; terminal states
    /// remain terminal even when the caller retains the returned token.
    pub fn consume<'binding>(
        &mut self,
        right: ConstructionRight,
        request: OperatorSigningRequest<'binding>,
        sign: impl FnOnce(
            OperatorSigningRequest<'binding>,
        )
            -> Result<OperatorAuthorizedCandidate<'binding>, OperatorSigningRefusal>,
    ) -> Result<OperatorRightOutcome<'binding>, Box<RightFailure>> {
        self.sign_once(&right, request, sign)
            .map_err(|refusal| Box::new(RightFailure { refusal, right }))
    }

    fn sign_once<'binding>(
        &mut self,
        right: &ConstructionRight,
        request: OperatorSigningRequest<'binding>,
        sign: impl FnOnce(
            OperatorSigningRequest<'binding>,
        )
            -> Result<OperatorAuthorizedCandidate<'binding>, OperatorSigningRefusal>,
    ) -> Result<OperatorRightOutcome<'binding>, RightRefusal> {
        let index = self.outstanding(right, false)?;
        let offered = RightScope::new(&request, right.scope.branch)?;
        if offered != right.scope {
            return Err(RightRefusal::ScopeMismatch {
                bound: Box::new(right.scope.clone()),
                offered: Box::new(offered),
            });
        }
        let entry = &self.entries[index];
        if entry.bytes != request.frozen_bytes() {
            return Err(RightRefusal::MismatchedBytes {
                right: entry.identity.clone(),
                bound: digest(&entry.bytes),
                offered: digest(request.frozen_bytes()),
            });
        }
        let authorized = match sign(request) {
            Ok(authorized) => authorized,
            Err(refusal) => {
                self.append(index, NonEquivocationEvent::SigningRefused);
                return Err(RightRefusal::Signing(Box::new(refusal)));
            }
        };
        let entry = &mut self.entries[index];
        if authorized.request().frozen_bytes() != entry.bytes
            || RightScope::new(authorized.request(), right.scope.branch)? != right.scope
        {
            entry.state = State::Indeterminate;
            let refusal = RightRefusal::UnexpectedAuthorization(entry.identity.clone());
            self.append(index, NonEquivocationEvent::UnexpectedAuthorization);
            return Err(refusal);
        }
        entry.cache = Some(CachedOperatorAuthorization {
            right: entry.identity.clone(),
            digest: digest(&entry.bytes),
            witness: authorized.witness().clone(),
            standing: authorized.standing().clone(),
        });
        entry.state = State::Signed;
        self.append(index, NonEquivocationEvent::Signed);
        Ok(OperatorRightOutcome::Fresh(Box::new(authorized)))
    }

    /// Reads an identical cached authorization without invoking a signer.
    ///
    /// # Errors
    /// Refuses unknown, stale, unsigned, indeterminate or competing candidates.
    pub fn retry(
        &mut self,
        scope: &RightScope,
        bytes: &[u8],
    ) -> Result<OperatorRightOutcome<'static>, RightRefusal> {
        let index = self.locate(scope)?;
        let entry = &self.entries[index];
        if entry.state == State::Indeterminate {
            return Err(RightRefusal::Indeterminate(entry.identity.clone()));
        }
        let cached = entry
            .cache
            .as_ref()
            .ok_or_else(|| RightRefusal::NotSigned(entry.identity.clone()))?;
        if bytes != entry.bytes {
            return Err(RightRefusal::CompetingCandidate {
                right: entry.identity.clone(),
                bound: digest(&entry.bytes),
                offered: digest(bytes),
            });
        }
        let outcome = OperatorRightOutcome::Cached(cached.clone());
        self.append(index, NonEquivocationEvent::Cached);
        Ok(outcome)
    }

    /// Marks an authorized candidate dispatched; repeated dispatch is a no-op.
    ///
    /// # Errors
    /// Refuses unknown, stale, unsigned or indeterminate scopes.
    pub fn dispatch(&mut self, scope: &RightScope) -> Result<(), RightRefusal> {
        let index = self.locate(scope)?;
        let entry = &mut self.entries[index];
        match entry.state {
            State::Signed => {
                entry.state = State::Dispatched;
            }
            State::Dispatched => return Ok(()),
            State::Indeterminate => {
                return Err(RightRefusal::Indeterminate(entry.identity.clone()));
            }
            _ => return Err(RightRefusal::NotSigned(entry.identity.clone())),
        }
        self.append(index, NonEquivocationEvent::Dispatched);
        Ok(())
    }

    /// Makes a dispatched right permanently indeterminate in this context.
    ///
    /// # Errors
    /// Refuses unknown, stale, undispatched or already indeterminate scopes.
    pub fn timeout(&mut self, scope: &RightScope) -> Result<(), RightRefusal> {
        let index = self.locate(scope)?;
        let entry = &mut self.entries[index];
        match entry.state {
            State::Dispatched => {
                entry.state = State::Indeterminate;
            }
            State::Indeterminate => {
                return Err(RightRefusal::Indeterminate(entry.identity.clone()));
            }
            _ => return Err(RightRefusal::NotDispatched(entry.identity.clone())),
        }
        self.append(index, NonEquivocationEvent::TimedOut);
        Ok(())
    }

    /// Invalidates every scope in a branch context and discards its caches.
    ///
    /// The tombstone also refuses scopes not yet issued in that context.
    /// Repeating invalidation changes nothing. Continuing requires a different,
    /// newly evidenced context; this method cannot establish that evidence.
    pub fn reorganize(&mut self, branch: &BranchContext) {
        if self.stale.contains(branch) {
            return;
        }
        self.stale.push(*branch);
        for entry in &mut self.entries {
            if entry.identity.scope.branch() == branch {
                entry.state = State::Invalidated;
                entry.cache = None;
                self.record
                    .entries
                    .push(observation(entry, NonEquivocationEvent::Invalidated));
            }
        }
    }

    /// The immutable audit record, never the signature-bearing cache.
    #[must_use]
    pub const fn record(&self) -> &NonEquivocationRecord {
        &self.record
    }

    fn check_branch(&self, branch: &BranchContext) -> Result<(), RightRefusal> {
        if self.stale.contains(branch) {
            Err(RightRefusal::StaleBranch(*branch))
        } else {
            Ok(())
        }
    }

    fn locate(&self, scope: &RightScope) -> Result<usize, RightRefusal> {
        self.check_branch(scope.branch())?;
        self.entries
            .iter()
            .position(|entry| entry.identity.scope.as_ref() == scope)
            .ok_or_else(|| RightRefusal::NoRight(Box::new(scope.clone())))
    }

    fn outstanding(
        &self,
        right: &ConstructionRight,
        replacing: bool,
    ) -> Result<usize, RightRefusal> {
        let index = self.locate(&right.scope)?;
        let entry = &self.entries[index];
        match entry.state {
            State::Outstanding => {}
            State::Indeterminate => {
                return Err(RightRefusal::Indeterminate(entry.identity.clone()));
            }
            _ if replacing => {
                return Err(RightRefusal::ReplacementAfterSigning(
                    entry.identity.clone(),
                ));
            }
            _ => return Err(RightRefusal::Consumed(entry.identity.clone())),
        }
        if !Arc::ptr_eq(&right.registry, &self.identity) || right.ordinal != index {
            return Err(RightRefusal::ForeignRight(entry.identity.clone()));
        }
        Ok(index)
    }

    fn append(&mut self, index: usize, event: NonEquivocationEvent) {
        self.record
            .entries
            .push(observation(&self.entries[index], event));
    }
}

fn observation(entry: &Entry, event: NonEquivocationEvent) -> NonEquivocationEntry {
    NonEquivocationEntry {
        right: entry.identity.clone(),
        event,
        bytes_len: entry.bytes.len(),
        digest: digest(&entry.bytes),
    }
}

fn digest(bytes: &[u8]) -> Digest32 {
    tagged_hash("operator-right/candidate", bytes)
}
