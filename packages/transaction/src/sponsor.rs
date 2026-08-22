//! The sponsor capability, and the signing requests it answers (§15.7).
//!
//! # Signing is a request, not a call into a key
//!
//! Nothing in this package holds, derives, accepts, or reads a private
//! key. A sponsor is an *adapter* the caller supplies, and the whole
//! interface between it and this crate is: here is the finalized
//! transaction, here is the input, here is the role and the sighash
//! profile, and here is the set of outputs your signature must protect
//! — return a witness or return nothing. A refusal to sign is an
//! ordinary outcome and not an error condition.
//!
//! # The request binds a finalized transaction, not a template
//!
//! §15.8 puts the signing request at stage twelve, after every
//! protected output is final, and §15.7 says a request binds the exact
//! finalized transaction. Both are structural here rather than
//! documented: a [`SponsorSigningRequest`] cannot be built without the
//! exact bytes it is about, and the returned signature carries the
//! bytes it was produced against so that the two can be compared by
//! exact byte comparison rather than by trust.
//!
//! # Sponsor-private values live here and are erased downstream
//!
//! A sponsor's own amounts are real, and constructing a balanced
//! transaction needs them. They enter through [`SponsorOffer`], are
//! consumed by the builder, and do not reach the construction report:
//! §1.6 forbids a canonical report from carrying an individual sponsor
//! amount, and the report type simply has no field for one.

use std::collections::BTreeSet;

use crate::bytes::{Outpoint, TargetOutput, ValueField};
use crate::error::TransactionRefusal;

/// Which sighash profile a signing request selects.
///
/// One variant, and named rather than assumed. A profile that committed
/// to fewer outputs would let a signature be replayed against a
/// transaction whose protected outputs differ, and §12.9's sponsor
/// isolation rests on that not being possible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SighashProfile {
    /// Commits to every input and every output of the transaction.
    ///
    /// Provenance: the default `SIGHASH_ALL` behaviour of the target's
    /// signature checkers, whose all-outputs mode commits every output
    /// and every output witness.
    AllInputsAllOutputs,
}

/// The public role a signer plays.
///
/// Public by construction: this is what a canonical report may retain
/// about a sponsor, and it names a position rather than an identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SignerRole {
    /// A member of the isolated sponsor suffix.
    SponsorSuffixMember,
}

/// What a sponsor adapter offers a construction.
///
/// The private half of the sponsor relationship. The amounts here are
/// sponsor-local: they are used to balance the transaction and are not
/// carried into any report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SponsorOffer {
    inputs: BTreeSet<Outpoint>,
    fee: u64,
    change: Option<ValueField>,
}

impl SponsorOffer {
    /// The offer of these inputs, this fee, and this optional change.
    ///
    /// A change of `None` states that the sponsor wants no change
    /// output. A change of `Some(ValueField::Explicit(0))` states a
    /// residual that happens to be zero, which the builder omits rather
    /// than emits — the target refuses a spendable zero-valued output,
    /// so emitting it would produce a transaction consensus rejects.
    ///
    /// Duplicates are rejected before sorting rather than collapsed by
    /// it, which is what [`crate::request::CompactAshRequest::new`]
    /// does with the ASH selection and for the same reason: an offer
    /// built by insertion would turn a sponsor naming one coin twice
    /// into a one-input offer nobody stated.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::DuplicateSponsorOutpoint`] when one
    /// outpoint is named more than once.
    pub fn new(
        inputs: impl IntoIterator<Item = Outpoint>,
        fee: u64,
        change: Option<ValueField>,
    ) -> Result<Self, TransactionRefusal> {
        let mut offered = BTreeSet::new();
        for outpoint in inputs {
            if !offered.insert(outpoint) {
                return Err(TransactionRefusal::DuplicateSponsorOutpoint(outpoint));
            }
        }
        Ok(Self {
            inputs: offered,
            fee,
            change,
        })
    }

    /// The outpoints the sponsor contributes.
    #[must_use]
    pub const fn inputs(&self) -> &BTreeSet<Outpoint> {
        &self.inputs
    }

    /// The fee the sponsor declares it is paying.
    #[must_use]
    pub const fn fee(&self) -> u64 {
        self.fee
    }

    /// The change the sponsor wants back, if any.
    #[must_use]
    pub const fn change(&self) -> Option<ValueField> {
        self.change
    }
}

/// One request for a sponsor to sign one input (§15.7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SponsorSigningRequest {
    transaction: Vec<u8>,
    input: u16,
    role: SignerRole,
    profile: SighashProfile,
    protected: Vec<TargetOutput>,
}

impl SponsorSigningRequest {
    /// The exact finalized transaction bytes the signature is about.
    ///
    /// Finalized: every output is already what it will be, and the only
    /// thing still to change is the witness of this and other inputs.
    #[must_use]
    pub fn transaction(&self) -> &[u8] {
        &self.transaction
    }

    /// Which input the signature is for.
    #[must_use]
    pub const fn input(&self) -> u16 {
        self.input
    }

    /// The public role of the signer being asked.
    #[must_use]
    pub const fn role(&self) -> SignerRole {
        self.role
    }

    /// The sighash profile the request selects.
    #[must_use]
    pub const fn profile(&self) -> SighashProfile {
        self.profile
    }

    /// The outputs the signature must protect.
    #[must_use]
    pub fn protected(&self) -> &[TargetOutput] {
        &self.protected
    }

    /// Build a request. Crate-internal, so stage twelve is the only
    /// place one can come from.
    pub(crate) const fn new(
        transaction: Vec<u8>,
        input: u16,
        role: SignerRole,
        profile: SighashProfile,
        protected: Vec<TargetOutput>,
    ) -> Self {
        Self {
            transaction,
            input,
            role,
            profile,
            protected,
        }
    }
}

/// What a sponsor adapter returns for one signing request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SponsorSignature {
    bound_to: Vec<u8>,
    stack: Vec<Vec<u8>>,
}

impl SponsorSignature {
    /// The witness `stack`, produced against exactly `bound_to`.
    ///
    /// The adapter echoes the transaction it signed rather than
    /// asserting that it signed the right one. The builder compares the
    /// echo with what it sent, byte for byte, which is the comparison
    /// §1.10 names for exact transaction bytes and is the only way an
    /// out-of-process signer's binding can be checked at all.
    #[must_use]
    pub const fn new(bound_to: Vec<u8>, stack: Vec<Vec<u8>>) -> Self {
        Self { bound_to, stack }
    }

    /// The transaction bytes the signature was produced against.
    #[must_use]
    pub fn bound_to(&self) -> &[u8] {
        &self.bound_to
    }

    /// The witness stack, bottom item first.
    #[must_use]
    pub fn stack(&self) -> &[Vec<u8>] {
        &self.stack
    }
}

/// A sponsor's external signing capability (§15.7).
///
/// A trait because the capability is genuinely outside this workspace:
/// a wallet, a hardware signer, a remote service. The three methods are
/// the whole of what this crate needs, and none of them hands a key in.
pub trait SponsorCapability {
    /// What this sponsor contributes to the construction.
    fn offer(&self) -> SponsorOffer;

    /// Where the sponsor wants its change, as a witness program
    /// payload and version.
    ///
    /// Checked against the deployment's own sponsor-change program
    /// before it is used. The candidate's emitted coordinator compares
    /// the change output's program against a deployment constant, so a
    /// destination that differs from it is refused rather than honoured
    /// — the destination is a sponsor's choice among what the
    /// deployment admits, which here is one program.
    fn change_destination(&self) -> Option<(u8, Vec<u8>)>;

    /// Sign one input of a finalized transaction, or decline.
    fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature>;
}

/// How many witness items the admitted sponsor program class takes.
///
/// Provenance: the version-zero key-hash branch of
/// `VerifyWitnessProgram` (`src/script/interpreter.cpp`), which refuses
/// a stack that is not exactly the signature and the public key.
pub const WITNESS_V0_KEYHASH_STACK_ITEMS: usize = 2;

/// How many bytes the admitted sponsor program class's payload
/// occupies.
///
/// Provenance: `WITNESS_V0_KEYHASH_SIZE` in the same file.
pub const WITNESS_V0_KEYHASH_PROGRAM_BYTES: usize = 20;
