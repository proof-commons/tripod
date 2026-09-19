//! The public current-STATE view (§12.4).
//!
//! # What the view is
//!
//! Ten public facts about one current STATE output, supplied by whoever
//! wants a maturity announcement built. Every one of them is a fact a
//! target node hands out to anybody or a deployment publishes, which is
//! what keeps an announcement permissionless to construct: there is no
//! operator secret here, no scalar, and no field one could be put in.
//!
//! What the view is *not* is an assertion this layer believes. A caller
//! states the ten entries; this module checks the one relation among them
//! that can be checked from the entries alone, and names what it did not
//! check rather than leaving a reader to assume it was checked.
//!
//! # The two laws
//!
//! Duplicate or contradictory views reject, and later entries cannot
//! overwrite earlier ones. Seven entries are stated, and a second
//! statement of any of them is refused before the view is assembled
//! rather than resolved by assembling it — the compact view's law, for
//! the compact view's reason: believing either of two contradictory
//! statements is believing the order the caller listed them in. There is
//! no setter, so nothing can overwrite an entry afterwards either.
//!
//! The remaining three entries are not stated at all. The current cycle
//! is the cycle the predecessor metadata carries, and the operator
//! public identity and the deployment symbols are what the accepted
//! bundle carries; each is read through an accessor of its own, so
//! §12.4's ten entries are all readable while one fact never has two
//! statements that could disagree. Refusing a disagreement would have
//! been the weaker arrangement: this view's only check is the
//! reconstruction below, and nothing here observes a chain, so a tie
//! between two statements of one fact would have nothing to break it.
//!
//! # The one check
//!
//! Encode the supplied predecessor metadata canonically with the supplied
//! representation nonce, commit the result under the accepted bundle's
//! own constructor policy over the linked static subtree, and compare the
//! program that commitment produces with the supplied predecessor
//! program. That equation is the announcement leaf's own: the leaf binds
//! the predecessor program to the tweak over the internal key and the
//! root derived from the authenticated metadata, and a first party can
//! compute it before any spend exists. Three of the caller's entries
//! either agree about one output or they do not, and this is where they
//! are made to say so instead of being taken on the caller's word.
//!
//! The nonce is committed as the caller named it. Searching for a nonce
//! that would make the comparison succeed would answer a different
//! question — which representations *could* have produced this program —
//! and the search over representations has one owner elsewhere, with its
//! own budget and its own leastness evidence.
//!
//! # The residual
//!
//! Freshness is not established here and is named for exactly that
//! reason: that the supplied outpoint is the *current* root is a fact
//! about a chain, no layer in this crate observes a chain, and the
//! structural record already declares it as an external evidence role.
//! Establishing it would need the branch-indexed history this crate does
//! not carry. So a validated view carries one residual, and a reader who
//! wants freshness knows from the type that it has to come from
//! somewhere else. The linked bundle's own obligation is unchanged by
//! this: what a link owes is a fact about that link, not a record of
//! what a later layer computed.

use std::collections::BTreeSet;

use linker::live_backend::OperatorKey;
use linker::{CandidateLinkedMaturityBundle, LinkedArtifactStatus, StateResolvedCensus};
use realization::{Cycle, EncodedStateMetadata, StateMetadata, StateRepresentationNonce};
use tapscript::{StateCurveCapability, state_output_program_at_nonce};
use target_elements::ReviewedElementsTapscriptDefinition;

use crate::bytes::{AssetField, Outpoint, ValueField};
use crate::error::TransactionRefusal;
use crate::operator_right::BranchContext;

/// One entry of the view's census, named so a refusal can name it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaturityViewEntry {
    /// The outpoint the current STATE output sits at.
    CurrentStateOutpoint,
    /// The exact asset and amount that output carries.
    AssetAndAmount,
    /// The canonical predecessor semantic metadata.
    PredecessorMetadata,
    /// The predecessor's representation nonce.
    PredecessorRepresentationNonce,
    /// The branch and checkpoint the current root is bound at.
    CurrentRootBinding,
    /// The predecessor output's actual program.
    PredecessorProgram,
    /// The current cycle ordinal.
    CurrentCycle,
    /// The accepted linked maturity bundle.
    AcceptedLinkedBundle,
    /// The operator's committed public identity.
    OperatorPublicIdentity,
    /// The deployment's resolved symbols.
    DeploymentSymbols,
}

impl MaturityViewEntry {
    /// Every entry, in the order §12.4 lists them.
    pub const ALL: &'static [Self] = &[
        Self::CurrentStateOutpoint,
        Self::AssetAndAmount,
        Self::PredecessorMetadata,
        Self::PredecessorRepresentationNonce,
        Self::CurrentRootBinding,
        Self::PredecessorProgram,
        Self::CurrentCycle,
        Self::AcceptedLinkedBundle,
        Self::OperatorPublicIdentity,
        Self::DeploymentSymbols,
    ];

    /// The entry this one is read from, for the three that are not
    /// stated.
    ///
    /// The current cycle is a field of the predecessor metadata — a STATE
    /// record states its cycle there, and the announcement window is
    /// derived from that field — while the operator public identity and
    /// the deployment symbols are held by the accepted bundle, which
    /// carries the deployment binding and the resolved census. A separate
    /// statement of any of the three would put one fact in two places,
    /// and this view has nothing with which to settle a disagreement
    /// between them. Reading them through their source instead makes
    /// "later entries cannot overwrite earlier ones" structural for these
    /// three and a refusal for the seven that are stated.
    #[must_use]
    pub const fn projected_from(self) -> Option<Self> {
        match self {
            Self::CurrentCycle => Some(Self::PredecessorMetadata),
            Self::OperatorPublicIdentity | Self::DeploymentSymbols => {
                Some(Self::AcceptedLinkedBundle)
            }
            Self::CurrentStateOutpoint
            | Self::AssetAndAmount
            | Self::PredecessorMetadata
            | Self::PredecessorRepresentationNonce
            | Self::CurrentRootBinding
            | Self::PredecessorProgram
            | Self::AcceptedLinkedBundle => None,
        }
    }

    /// The stable diagnostic name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CurrentStateOutpoint => "current-state-outpoint",
            Self::AssetAndAmount => "asset-and-amount",
            Self::PredecessorMetadata => "predecessor-metadata",
            Self::PredecessorRepresentationNonce => "predecessor-representation-nonce",
            Self::CurrentRootBinding => "current-root-binding",
            Self::PredecessorProgram => "predecessor-program",
            Self::CurrentCycle => "current-cycle",
            Self::AcceptedLinkedBundle => "accepted-linked-bundle",
            Self::OperatorPublicIdentity => "operator-public-identity",
            Self::DeploymentSymbols => "deployment-symbols",
        }
    }
}

/// One caller's statement of one stated entry.
///
/// A stream of statements rather than a positional constructor, because
/// the duplicate law has content only where statements arrive one after
/// another: ten parameters would make a second statement unrepresentable
/// by making a silently wrong *order* representable in its place, and the
/// compact view refuses a second statement for the same reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityViewStatement {
    /// The outpoint the current STATE output sits at.
    CurrentStateOutpoint(Outpoint),
    /// The exact asset and amount that output carries, as one entry.
    ///
    /// One statement and not two, because §12.4 lists them as one entry:
    /// an amount without the asset it is denominated in is not a fact
    /// about an output.
    AssetAndAmount(AssetField, ValueField),
    /// The canonical predecessor semantic metadata.
    PredecessorMetadata(StateMetadata),
    /// The predecessor's representation nonce.
    PredecessorRepresentationNonce(StateRepresentationNonce),
    /// The branch and checkpoint the current root is bound at.
    CurrentRootBinding(BranchContext),
    /// The predecessor output's actual program.
    PredecessorProgram(Vec<u8>),
    /// The accepted linked maturity bundle.
    ///
    /// Boxed because a linked bundle is far larger than any other
    /// statement, and an enumeration is as large as its largest variant.
    AcceptedLinkedBundle(Box<CandidateLinkedMaturityBundle>),
}

impl MaturityViewStatement {
    /// Which entry this statement states.
    #[must_use]
    pub const fn entry(&self) -> MaturityViewEntry {
        match self {
            Self::CurrentStateOutpoint(..) => MaturityViewEntry::CurrentStateOutpoint,
            Self::AssetAndAmount(..) => MaturityViewEntry::AssetAndAmount,
            Self::PredecessorMetadata(..) => MaturityViewEntry::PredecessorMetadata,
            Self::PredecessorRepresentationNonce(..) => {
                MaturityViewEntry::PredecessorRepresentationNonce
            }
            Self::CurrentRootBinding(..) => MaturityViewEntry::CurrentRootBinding,
            Self::PredecessorProgram(..) => MaturityViewEntry::PredecessorProgram,
            Self::AcceptedLinkedBundle(..) => MaturityViewEntry::AcceptedLinkedBundle,
        }
    }
}

/// What a validated view did not establish.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum MaturityViewResidual {
    /// Nothing here establishes that the outpoint is the current root.
    ///
    /// The reconstruction shows that the supplied metadata, nonce and
    /// program describe one output; it says nothing about whether that
    /// output is still unspent, still the thread's tip, or on the branch
    /// the caller believes. Those are facts about a chain. This crate
    /// observes none, the structural record declares freshness as an
    /// external evidence role rather than a checked one, and a check here
    /// would need the branch-indexed history this crate does not carry.
    /// Naming the gap is the whole of what this layer can honestly do
    /// about it: an unnamed residual reads as a discharged one.
    CurrentStateRootFreshness,
}

/// The public current-STATE view one announcement is built from.
///
/// A stated entry cannot be overwritten, because there is nothing to
/// overwrite it with:
///
/// ```compile_fail,E0599
/// use transaction::{Outpoint, PublicMaturityStateView};
/// fn overwrite(view: &mut PublicMaturityStateView, outpoint: Outpoint) {
///     view.set_current_state_outpoint(outpoint);
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicMaturityStateView {
    outpoint: Outpoint,
    asset: AssetField,
    value: ValueField,
    metadata: StateMetadata,
    nonce: StateRepresentationNonce,
    root_binding: BranchContext,
    program: Vec<u8>,
    bundle: CandidateLinkedMaturityBundle,
}

impl PublicMaturityStateView {
    /// The view these statements state.
    ///
    /// Each stated entry exactly once, in any order. A second statement
    /// of one entry is refused before anything is assembled, so no
    /// statement can be overwritten by a later one and no refused view
    /// exists to be read; an entry never stated is refused rather than
    /// defaulted, because a default would be a fact this layer invented
    /// and then read back as the caller's.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::DuplicateMaturityViewEntry`] when one entry
    /// is stated more than once, whether the two statements agree or
    /// contradict, and
    /// [`TransactionRefusal::MissingMaturityViewEntry`] when a stated
    /// entry is absent.
    pub fn new(
        statements: impl IntoIterator<Item = MaturityViewStatement>,
    ) -> Result<Self, TransactionRefusal> {
        let mut seen = BTreeSet::new();
        let mut outpoint = None;
        let mut asset_and_amount = None;
        let mut metadata = None;
        let mut nonce = None;
        let mut root_binding = None;
        let mut program = None;
        let mut bundle = None;

        for statement in statements {
            let entry = statement.entry();
            if !seen.insert(entry) {
                return Err(TransactionRefusal::DuplicateMaturityViewEntry(entry));
            }
            match statement {
                MaturityViewStatement::CurrentStateOutpoint(stated) => outpoint = Some(stated),
                MaturityViewStatement::AssetAndAmount(asset, value) => {
                    asset_and_amount = Some((asset, value));
                }
                MaturityViewStatement::PredecessorMetadata(stated) => metadata = Some(stated),
                MaturityViewStatement::PredecessorRepresentationNonce(stated) => {
                    nonce = Some(stated);
                }
                MaturityViewStatement::CurrentRootBinding(stated) => root_binding = Some(stated),
                MaturityViewStatement::PredecessorProgram(stated) => program = Some(stated),
                MaturityViewStatement::AcceptedLinkedBundle(stated) => bundle = Some(*stated),
            }
        }

        let absent = TransactionRefusal::MissingMaturityViewEntry;
        let (asset, value) =
            asset_and_amount.ok_or_else(|| absent(MaturityViewEntry::AssetAndAmount))?;
        Ok(Self {
            outpoint: outpoint.ok_or_else(|| absent(MaturityViewEntry::CurrentStateOutpoint))?,
            asset,
            value,
            metadata: metadata.ok_or_else(|| absent(MaturityViewEntry::PredecessorMetadata))?,
            nonce: nonce
                .ok_or_else(|| absent(MaturityViewEntry::PredecessorRepresentationNonce))?,
            root_binding: root_binding
                .ok_or_else(|| absent(MaturityViewEntry::CurrentRootBinding))?,
            program: program.ok_or_else(|| absent(MaturityViewEntry::PredecessorProgram))?,
            bundle: bundle.ok_or_else(|| absent(MaturityViewEntry::AcceptedLinkedBundle))?,
        })
    }

    /// The outpoint the current STATE output sits at.
    #[must_use]
    pub const fn current_state_outpoint(&self) -> Outpoint {
        self.outpoint
    }

    /// The exact asset field that output carries.
    #[must_use]
    pub const fn asset(&self) -> AssetField {
        self.asset
    }

    /// The exact amount field that output carries.
    #[must_use]
    pub const fn value(&self) -> ValueField {
        self.value
    }

    /// The canonical predecessor semantic metadata.
    #[must_use]
    pub const fn predecessor_metadata(&self) -> StateMetadata {
        self.metadata
    }

    /// The predecessor's representation nonce.
    #[must_use]
    pub const fn predecessor_representation_nonce(&self) -> StateRepresentationNonce {
        self.nonce
    }

    /// The branch and checkpoint the current root is bound at.
    ///
    /// A [`BranchContext`] and not a thread anchor. An anchor states
    /// where a thread *began*, together with provenance, recipe
    /// generation, checkpoint policy and origin — none of which §12.4
    /// lists, and its starting outpoint would restate this view's first
    /// entry, giving two entries room to disagree about one outpoint. The
    /// branch context is the whole of the binding: which branch
    /// identifier and which checkpoint ordinal the caller binds this root
    /// at, established by the caller externally and checked here only for
    /// shape, which is exactly the shape of the residual below.
    #[must_use]
    pub const fn current_root_binding(&self) -> BranchContext {
        self.root_binding
    }

    /// The predecessor output's actual program.
    #[must_use]
    pub fn predecessor_program(&self) -> &[u8] {
        &self.program
    }

    /// The current cycle ordinal.
    ///
    /// Read from the predecessor metadata, which is where a STATE record
    /// states its cycle.
    #[must_use]
    pub const fn current_cycle(&self) -> Cycle {
        self.metadata.cycle
    }

    /// The accepted linked maturity bundle.
    #[must_use]
    pub const fn accepted_linked_bundle(&self) -> &CandidateLinkedMaturityBundle {
        &self.bundle
    }

    /// The operator's committed public identity.
    ///
    /// The key the accepted bundle's deployment binding committed, read
    /// through that binding rather than restated beside it.
    #[must_use]
    pub const fn operator_public_identity(&self) -> &OperatorKey {
        self.bundle.deployment().operator().key()
    }

    /// The deployment's resolved symbols.
    ///
    /// The bundle's resolved census: every symbol the link defined, its
    /// definition, and where the composed program consumes it.
    #[must_use]
    pub const fn deployment_symbols(&self) -> &StateResolvedCensus {
        self.bundle.resolved()
    }

    /// Check what the entries say about each other, and name what stays
    /// open.
    ///
    /// The reviewed target and the curve capability are arguments rather
    /// than entries because neither is a public fact about an output: the
    /// target supplies the metadata leaf's encoding and its leaf version,
    /// and curve arithmetic is a behaviour a caller provides, which is
    /// why the constructor takes it from a capability instead of
    /// computing it.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::BundleIsNotACandidate`] for a bundle
    /// claiming more than a prototype link,
    /// [`TransactionRefusal::ContractRevisionMismatch`] when the bundle's
    /// policy and the target disagree about the reviewed revision,
    /// [`TransactionRefusal::MaturityViewCommitmentRefused`] when the
    /// supplied metadata and nonce commit to nothing at that nonce, and
    /// [`TransactionRefusal::MaturityViewProgramNotReconstructed`] when
    /// they commit to a program other than the supplied one.
    pub fn validate(
        &self,
        target: &ReviewedElementsTapscriptDefinition,
        curve: &impl StateCurveCapability,
    ) -> Result<ValidatedMaturityStateView, TransactionRefusal> {
        if self.bundle.status() != LinkedArtifactStatus::Prototype {
            return Err(TransactionRefusal::BundleIsNotACandidate);
        }
        if self.bundle.policy().target_policy() != target.definition().version() {
            return Err(TransactionRefusal::ContractRevisionMismatch);
        }

        let encoded = EncodedStateMetadata {
            semantic: self.metadata,
            representation: self.nonce,
        };
        let recomputed = state_output_program_at_nonce(
            target,
            &encoded,
            self.bundle.static_subtree(),
            self.bundle.policy().internal_key(),
            curve,
        )
        .map_err(|refusal| TransactionRefusal::MaturityViewCommitmentRefused { refusal })?;

        if recomputed == self.program {
            Ok(ValidatedMaturityStateView { view: self.clone() })
        } else {
            Err(TransactionRefusal::MaturityViewProgramNotReconstructed {
                supplied: self.program.len(),
                recomputed: recomputed.len(),
            })
        }
    }
}

/// A view whose one checkable relation has been computed.
///
/// The type is the evidence: it exists only where the supplied metadata
/// and nonce were committed under the accepted bundle's policy and
/// reproduced the supplied program, and there is no constructor through
/// which a caller could claim that without it having happened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedMaturityStateView {
    view: PublicMaturityStateView,
}

impl ValidatedMaturityStateView {
    /// The ten entries, read through the view that was checked.
    ///
    /// One accessor rather than ten delegating copies: a second census
    /// would be a second place for an entry to drift, and the entries
    /// worth exposing here are exactly the entries the check ran over.
    #[must_use]
    pub const fn view(&self) -> &PublicMaturityStateView {
        &self.view
    }

    /// What validation did not establish.
    #[must_use]
    pub const fn residual(&self) -> MaturityViewResidual {
        MaturityViewResidual::CurrentStateRootFreshness
    }
}
