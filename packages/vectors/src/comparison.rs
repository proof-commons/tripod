//! Reading an accepted target transaction back into §17.4's projection.
//!
//! §1.4 makes target acceptance and semantic agreement two verdicts, and
//! §17.4 lists the thirteen terms the second one compares. This module
//! is that comparison: it reads a projection off the transaction the
//! target accepted and off what the target said about the coins that
//! transaction spends, and puts it beside the projection the realization
//! layer derived before anything ran.
//!
//! # Why the observed side is its own type
//!
//! [`AcceptedProjection`] fixes five of its terms in its constructor, so
//! that an expectation cannot be weakened into agreeing with whatever
//! turned up. That is right for an expectation and wrong for an
//! observation: a side that cannot say "this transaction carried an
//! issuance" cannot disagree about issuance, and five of the thirteen
//! comparisons would pass by construction. [`ObservedProjection`]
//! therefore states every term freely, and the two types meet only in
//! [`compare`].
//!
//! # What the transaction decides and what the report contributes
//!
//! Eleven terms are read off the accepted bytes and the target's own
//! answers. Two are not, and are named here rather than smuggled: the
//! transition-certificate derivation is a fact about how this comparison
//! was made, and the operation identity is the operation the plan ran.
//! Neither is discoverable in a transaction, and a comparison that
//! pretended otherwise would be reporting its own inputs as findings.
//!
//! # Why the input family is compared as a family
//!
//! §18.1 names canonical input-order normalization as a positive class,
//! and the ABI reaches it by ordering the ASH inputs by outpoint. The
//! order a target puts them in is therefore not the order the fixture
//! states them in, by design. The semantic handles are the fixture's own
//! numbering and are not target facts at all, so what is compared is the
//! multiset of amounts and its cardinality. Comparing positions would
//! fail the very class §18.1 asks for, and comparing handles would
//! compare the fixture's numbering with itself.

use std::collections::{BTreeMap, BTreeSet};

use transaction::{AssetField, AssetId, NonceField, Outpoint, TargetTransaction, ValueField};

use crate::projection::{
    AcceptedProjection, CanonicalFlowKind, DestructionClaim, EventClaim, IssuanceClaim,
    OwnershipStatus, RootSuccession, SponsorRegion, TransitionCertificate,
};

/// Why an accepted transaction could not be read at all.
///
/// Distinct from a disagreement. A transaction whose bytes do not decode
/// has not disagreed with the model about anything; it has failed to be
/// a transaction, and recording that as a semantic mismatch would file a
/// transport or construction defect as a protocol finding.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProjectionRefusal {
    /// The accepted bytes are not a transaction this workspace decodes.
    Undecodable,
    /// The transaction spends nothing.
    NoInputs,
    /// The transaction creates nothing.
    NoOutputs,
    /// No output carries the asset the ceremony issued.
    NoSuccessorOutput,
    /// More than one output carries it, so which is the successor is
    /// not decidable and no reading may be guessed.
    AmbiguousSuccessor(usize),
    /// The successor output's value is a commitment rather than an
    /// amount, so there is no amount to compare.
    SuccessorValueIsConfidential,
}

/// One §17.4 term.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ProjectionTerm {
    /// The exact ASH input family.
    InputFamily,
    /// The exact successor ASH family member.
    Successor,
    /// The exact explicit `U`.
    ExplicitU,
    /// The exact aggregate amount.
    Aggregate,
    /// The ownerless status.
    Ownership,
    /// The absence of root succession.
    Roots,
    /// The absence of issuance.
    Issuance,
    /// The absence of destruction.
    Destruction,
    /// The canonical movement kind.
    Flow,
    /// The sponsor region's existence and membership.
    Sponsor,
    /// The absence of a specialized event.
    Event,
    /// The transition-certificate derivation.
    Certificate,
    /// The exact operation identity.
    Operation,
}

impl ProjectionTerm {
    /// Every term §17.4 lists, in its own order.
    pub const ALL: &'static [Self] = &[
        Self::InputFamily,
        Self::Successor,
        Self::ExplicitU,
        Self::Aggregate,
        Self::Ownership,
        Self::Roots,
        Self::Issuance,
        Self::Destruction,
        Self::Flow,
        Self::Sponsor,
        Self::Event,
        Self::Certificate,
        Self::Operation,
    ];

    /// Its stable name, for a report.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::InputFamily => "input-family",
            Self::Successor => "successor",
            Self::ExplicitU => "explicit-u",
            Self::Aggregate => "aggregate",
            Self::Ownership => "ownership",
            Self::Roots => "roots",
            Self::Issuance => "issuance",
            Self::Destruction => "destruction",
            Self::Flow => "flow",
            Self::Sponsor => "sponsor",
            Self::Event => "event",
            Self::Certificate => "certificate",
            Self::Operation => "operation",
        }
    }
}

/// The projection an accepted transaction actually exhibits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedProjection {
    input_family: Vec<u64>,
    successor: u64,
    explicit_u: u64,
    aggregate: u64,
    ownership: OwnershipStatus,
    roots: RootSuccession,
    issuance: IssuanceClaim,
    destruction: DestructionClaim,
    flow: CanonicalFlowKind,
    sponsor: SponsorRegion,
    event: EventClaim,
    certificate: TransitionCertificate,
    operation: &'static str,
}

impl ObservedProjection {
    /// The amounts of the coins the transaction's ASH inputs spend.
    #[must_use]
    pub fn input_family(&self) -> &[u64] {
        &self.input_family
    }

    /// The successor output's amount.
    #[must_use]
    pub const fn successor(&self) -> u64 {
        self.successor
    }

    /// The sponsor region the transaction exhibits.
    #[must_use]
    pub const fn sponsor(&self) -> SponsorRegion {
        self.sponsor
    }
}

/// How one output of an accepted transaction is classified.
enum OutputRole {
    /// It carries the protocol object: the successor.
    Successor,
    /// Its program is empty: the target's fee role, which is not a
    /// protocol object and carries nothing to project.
    Fee,
    /// Anything else, which no compact-ASH sponsorless shape defines.
    Foreign,
}

/// Recognize the successor by the asset it carries, never by where it
/// is paid.
///
/// # Why not by the program
///
/// Recognizing it by the constructor program would make the ownerless
/// status true by construction: the output would be the successor
/// *because* it sits behind the covenant, and the ownership comparison
/// would have nothing left to decide. The protocol object is the asset,
/// so the asset is what finds it, and where it was paid is then a fact
/// about that output which the comparison can disagree with.
fn classify(output: &transaction::TargetOutput, asset: AssetId) -> OutputRole {
    if output.asset() == AssetField::Explicit(asset) && !output.program().is_empty() {
        OutputRole::Successor
    } else if output.program().is_empty() {
        OutputRole::Fee
    } else {
        OutputRole::Foreign
    }
}

/// Read the projection an accepted transaction exhibits.
///
/// `coins` is what the target reported putting in each output the
/// ceremony created for this vector, `asset` is the identity the target
/// chose when it issued, and `constructor` is the program the ceremony
/// funded and the target confirmed storing. All three are the target's
/// own answers; nothing the fixture said enters here, which is what
/// makes the result something to compare rather than an echo.
///
/// # Errors
///
/// [`ProjectionRefusal`] where the bytes are not readable as a
/// transaction of the shape a projection is defined over. A refusal is
/// not a disagreement and must not be recorded as one.
pub fn read_accepted(
    bytes: &[u8],
    coins: &BTreeMap<Outpoint, u64>,
    asset: [u8; 32],
    constructor: &[u8],
    operation: &'static str,
) -> Result<ObservedProjection, ProjectionRefusal> {
    let decoded = TargetTransaction::decode(bytes).map_err(|_| ProjectionRefusal::Undecodable)?;
    if decoded.inputs().is_empty() {
        return Err(ProjectionRefusal::NoInputs);
    }
    if decoded.outputs().is_empty() {
        return Err(ProjectionRefusal::NoOutputs);
    }

    // An input spending a coin this ceremony cut for this vector is an
    // ASH input; anything else is a member of a sponsor region, since
    // the only other way a compact-ASH transaction takes value in is
    // through one. Counted rather than assumed absent.
    let mut family = Vec::with_capacity(decoded.inputs().len());
    let mut foreign_inputs = 0_u16;
    for input in decoded.inputs() {
        match coins.get(&input.outpoint()) {
            Some(amount) => family.push(*amount),
            None => foreign_inputs = foreign_inputs.saturating_add(1),
        }
    }

    let object = AssetId::from_internal(asset);
    let mut successor_output = None;
    let mut successors = 0_usize;
    let mut fee_outputs = 0_usize;
    let mut foreign_outputs = 0_usize;
    for output in decoded.outputs() {
        match classify(output, object) {
            OutputRole::Successor => {
                successors += 1;
                successor_output = Some(output);
            }
            OutputRole::Fee => fee_outputs += 1,
            OutputRole::Foreign => foreign_outputs += 1,
        }
    }
    if successors == 0 {
        return Err(ProjectionRefusal::NoSuccessorOutput);
    }
    if successors > 1 {
        return Err(ProjectionRefusal::AmbiguousSuccessor(successors));
    }
    let output = successor_output.ok_or(ProjectionRefusal::NoSuccessorOutput)?;
    let ValueField::Explicit(successor) = output.value() else {
        return Err(ProjectionRefusal::SuccessorValueIsConfidential);
    };

    // Where the object was paid, which is a fact about this output and
    // not a premise of having found it. Behind the covenant program the
    // ceremony funded, it answers to the operation's own rules and to
    // no owner; anywhere else, somebody holds it.
    let ownership = if output.program() == constructor {
        OwnershipStatus::Ownerless
    } else {
        OwnershipStatus::Owned
    };

    // An output no compact-ASH shape defines is the only thing in this
    // ABI that could carry a root, a destruction, or a specialized
    // event, so the three absences are read off the same fact and not
    // asserted separately. A nonce on the successor is likewise a
    // representation the projection is not defined over.
    let irregular = foreign_outputs > 0 || !matches!(output.nonce(), NonceField::Null);
    let roots = if irregular {
        RootSuccession::Present
    } else {
        RootSuccession::Absent
    };
    let destruction = if irregular {
        DestructionClaim::Present
    } else {
        DestructionClaim::Absent
    };
    let event = if irregular {
        EventClaim::Specialized
    } else {
        EventClaim::Absent
    };

    // The issuance claim is the decoder's: this workspace's transaction
    // type has no field an issuance could occupy, and bytes carrying one
    // are refused rather than represented, so a decoded transaction has
    // none. The absence is established by the decode above, and is
    // recorded here rather than left implicit.
    let issuance = IssuanceClaim::Absent;

    // A fee output means a sponsor paid one: the sponsorless form
    // carries none.
    let sponsor = if foreign_inputs == 0 && fee_outputs == 0 {
        SponsorRegion::ABSENT
    } else {
        SponsorRegion::present(foreign_inputs)
    };

    let flow = if ownership == OwnershipStatus::Ownerless
        && roots == RootSuccession::Absent
        && issuance == IssuanceClaim::Absent
        && destruction == DestructionClaim::Absent
    {
        CanonicalFlowKind::OwnerlessLateral
    } else {
        CanonicalFlowKind::OwnerControlledLateral
    };

    Ok(ObservedProjection {
        input_family: family,
        successor,
        // The accepted transaction states one number where the model
        // states three, and this is that number in each of the three
        // places. The comparison is still a comparison — the model's
        // three came from its own arithmetic over the fixture and this
        // one came off the wire — but the three tests are not
        // independent of each other, and saying so is the point of this
        // comment.
        explicit_u: successor,
        aggregate: successor,
        ownership,
        roots,
        issuance,
        destruction,
        flow,
        sponsor,
        event,
        // Contributed by this comparison rather than found in the
        // transaction. See the module documentation.
        certificate: TransitionCertificate::DerivedFromRelations,
        operation,
    })
}

fn multiset(amounts: impl IntoIterator<Item = u64>) -> BTreeMap<u64, usize> {
    let mut counted = BTreeMap::new();
    for amount in amounts {
        *counted.entry(amount).or_insert(0_usize) += 1;
    }
    counted
}

/// Compare the derived projection against the observed one.
///
/// Returns every §17.4 term that differed. An empty set is agreement on
/// all thirteen, which is the second of §1.4's two verdicts.
#[must_use]
pub fn compare(
    expected: &AcceptedProjection,
    observed: &ObservedProjection,
) -> BTreeSet<ProjectionTerm> {
    let mut differed = BTreeSet::new();

    let wanted = multiset(
        expected
            .input_family()
            .iter()
            .map(|(_, amount)| amount.get()),
    );
    if wanted != multiset(observed.input_family.iter().copied()) {
        differed.insert(ProjectionTerm::InputFamily);
    }
    if expected.successor().1.get() != observed.successor {
        differed.insert(ProjectionTerm::Successor);
    }
    if expected.explicit_u().get() != observed.explicit_u {
        differed.insert(ProjectionTerm::ExplicitU);
    }
    if expected.aggregate().get() != observed.aggregate {
        differed.insert(ProjectionTerm::Aggregate);
    }
    if expected.ownership() != observed.ownership {
        differed.insert(ProjectionTerm::Ownership);
    }
    if expected.roots() != observed.roots {
        differed.insert(ProjectionTerm::Roots);
    }
    if expected.issuance() != observed.issuance {
        differed.insert(ProjectionTerm::Issuance);
    }
    if expected.destruction() != observed.destruction {
        differed.insert(ProjectionTerm::Destruction);
    }
    if expected.flow() != observed.flow {
        differed.insert(ProjectionTerm::Flow);
    }
    if expected.sponsor() != observed.sponsor {
        differed.insert(ProjectionTerm::Sponsor);
    }
    if expected.event() != observed.event {
        differed.insert(ProjectionTerm::Event);
    }
    if expected.certificate() != observed.certificate {
        differed.insert(ProjectionTerm::Certificate);
    }
    if expected.operation() != observed.operation {
        differed.insert(ProjectionTerm::Operation);
    }

    differed
}

#[cfg(test)]
mod tests {
    use super::{ObservedProjection, ProjectionRefusal, ProjectionTerm, compare, read_accepted};
    use crate::bundle::fixture_bundle;
    use crate::fixture::{OPERATION, positive_semantic_census};
    use crate::materialize::{AshFunding, is_materializable, materialize, vector_id};
    use std::collections::BTreeMap;
    use transaction::Outpoint;

    /// One materialized vector, the coins it spends, and its program.
    struct Sample {
        bytes: Vec<u8>,
        coins: BTreeMap<Outpoint, u64>,
        asset: [u8; 32],
        program: Vec<u8>,
        expected: crate::projection::AcceptedProjection,
    }

    /// Build a sample from a named §18.1 class.
    ///
    /// The coins are the placeholder ones the canonical plan uses, which
    /// is exactly the situation this comparison is defined over: the
    /// amounts stand for what a target reported, and nothing here needs
    /// them to exist on a chain.
    fn sample(class: &str) -> Sample {
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let census = positive_semantic_census().expect("the positive census builds");
        let case = census
            .iter()
            .filter(|case| is_materializable(case))
            .find(|case| case.class().name() == class)
            .expect("the class is a sponsorless positive one");
        let funding = AshFunding::unexecutable_placeholder(vector_id(case));
        let coins = funding
            .outpoints()
            .iter()
            .copied()
            .zip(case.inputs().iter().map(|amount| amount.get()))
            .collect();
        let vector = materialize(&bundle, case, &funding).expect("it materializes");
        let program = bundle
            .pin()
            .output_script(bundle.target())
            .expect("the pinned program derives");
        Sample {
            bytes: vector.bytes().to_vec(),
            coins,
            asset: bundle.closed_asset(),
            program,
            expected: case.expected().clone(),
        }
    }

    fn read(sample: &Sample) -> ObservedProjection {
        read_accepted(
            &sample.bytes,
            &sample.coins,
            sample.asset,
            &sample.program,
            OPERATION,
        )
        .expect("the transaction reads back")
    }

    #[test]
    fn every_sponsorless_vector_projects_to_its_own_expectation() {
        // The comparison over the whole census, so that agreement is a
        // property of the construction rather than of one lucky row.
        let census = positive_semantic_census().expect("the positive census builds");
        let names: Vec<&str> = census
            .iter()
            .filter(|case| is_materializable(case))
            .map(|case| case.class().name())
            .collect();
        assert_eq!(names.len(), 9);

        for name in names {
            let sample = sample(name);
            let observed = read(&sample);
            let differed = compare(&sample.expected, &observed);
            assert!(
                differed.is_empty(),
                "{name} differed on {:?}",
                differed.iter().map(|term| term.name()).collect::<Vec<_>>()
            );
        }
    }

    /// The same row funded by coins whose order the ABI will reverse.
    ///
    /// The placeholder coins ascend with the member position, so under
    /// them the canonical order and the fixture's order coincide and a
    /// sequence comparison would pass by luck. These descend, which is
    /// what a ceremony produces in general: the coins arrive from
    /// separate transactions and their identities have nothing to do
    /// with the order the fixture states its amounts in.
    fn reversed_sample(class: &str) -> Sample {
        use transaction::Txid;

        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let census = positive_semantic_census().expect("the positive census builds");
        let case = census
            .iter()
            .filter(|case| is_materializable(case))
            .find(|case| case.class().name() == class)
            .expect("the class is a sponsorless positive one");
        let id = vector_id(case);

        let members = usize::from(id.ash_inputs());
        let outpoints: Vec<Outpoint> = (0..members)
            .map(|member| {
                let mut seed = [0_u8; 32];
                // Descending in the member position, so the ascending
                // outpoint order the ABI takes is the reverse of it.
                seed[31] = u8::try_from(members - member).unwrap_or(u8::MAX);
                Outpoint::new(Txid::from_internal(seed), 0).expect("the outpoint is admissible")
            })
            .collect();
        let coins = outpoints
            .iter()
            .copied()
            .zip(case.inputs().iter().map(|amount| amount.get()))
            .collect();
        let funding = AshFunding::new(id, outpoints).expect("the funding record is well formed");
        let vector = materialize(&bundle, case, &funding).expect("it materializes");
        Sample {
            bytes: vector.bytes().to_vec(),
            coins,
            asset: bundle.closed_asset(),
            program: bundle
                .pin()
                .output_script(bundle.target())
                .expect("the pinned program derives"),
            expected: case.expected().clone(),
        }
    }

    #[test]
    fn the_normalized_row_agrees_although_its_order_changed() {
        // §18.1's canonical input-order normalization class. The target
        // orders the ASH inputs by outpoint, so the family the
        // transaction exhibits is a permutation of the one the fixture
        // states, and the comparison has to be over the family rather
        // than over the sequence or this class could never pass.
        let sample = reversed_sample("canonical-input-order-normalization");
        let observed = read(&sample);
        let stated: Vec<u64> = sample
            .expected
            .input_family()
            .iter()
            .map(|(_, amount)| amount.get())
            .collect();
        assert_eq!(stated, vec![300, 200, 100]);
        assert_eq!(
            observed.input_family(),
            [100, 200, 300],
            "the ABI orders the family by outpoint, which is the whole class"
        );
        assert_ne!(
            observed.input_family(),
            stated.as_slice(),
            "this row exists because the orders differ; if they stopped differing it tests nothing"
        );
        assert!(compare(&sample.expected, &observed).is_empty());
    }

    #[test]
    fn a_reordered_family_still_disagrees_when_an_amount_changes() {
        // The permutation tolerance must not become blindness: the
        // multiset still has to discriminate once the coins are in an
        // order the fixture did not state.
        let mut sample = reversed_sample("canonical-input-order-normalization");
        let first = *sample.coins.keys().next().expect("a coin exists");
        sample.coins.insert(first, 999);
        let differed = compare(&sample.expected, &read(&sample));
        assert!(differed.contains(&ProjectionTerm::InputFamily));
    }

    #[test]
    fn a_changed_amount_is_caught_at_the_family_and_not_only_at_the_sum() {
        // The comparison must discriminate, or an empty difference set
        // would mean nothing. One coin worth one unit more, with the
        // successor left alone, is the case a sum-only test misses.
        let mut sample = sample("minimum-two-ash-inputs");
        let first = *sample.coins.keys().next().expect("a coin exists");
        sample.coins.insert(first, 121);
        let observed = read(&sample);
        let differed = compare(&sample.expected, &observed);
        assert!(differed.contains(&ProjectionTerm::InputFamily));
        assert!(!differed.contains(&ProjectionTerm::Successor));
    }

    #[test]
    fn a_coin_no_ceremony_cut_reads_as_a_sponsor_member() {
        // An input the ceremony did not fund is the only way value
        // enters a compact-ASH transaction other than through the ASH
        // family, so it is counted as a sponsor member rather than
        // ignored. A comparison that ignored it would call a sponsored
        // transaction sponsorless.
        let mut sample = sample("minimum-two-ash-inputs");
        let first = *sample.coins.keys().next().expect("a coin exists");
        sample.coins.remove(&first);
        let observed = read(&sample);
        assert!(observed.sponsor().is_present());
        assert_eq!(observed.sponsor().members(), 1);
        let differed = compare(&sample.expected, &observed);
        assert!(differed.contains(&ProjectionTerm::Sponsor));
        assert!(differed.contains(&ProjectionTerm::InputFamily));
    }

    #[test]
    fn bytes_that_are_not_a_transaction_are_refused_and_not_called_a_mismatch() {
        let sample = sample("minimum-two-ash-inputs");
        let refused = read_accepted(
            &[0x00, 0x01],
            &sample.coins,
            sample.asset,
            &sample.program,
            OPERATION,
        )
        .expect_err("two bytes are not a transaction");
        assert_eq!(refused, ProjectionRefusal::Undecodable);
    }

    #[test]
    fn a_transaction_carrying_some_other_asset_has_no_successor() {
        // The successor is the output carrying the protocol object, so
        // a reading against an identity no output carries must fail
        // rather than pick whichever output looks plausible.
        let sample = sample("minimum-two-ash-inputs");
        let refused = read_accepted(
            &sample.bytes,
            &sample.coins,
            [0xcd; 32],
            &sample.program,
            OPERATION,
        )
        .expect_err("no output carries that asset");
        assert_eq!(refused, ProjectionRefusal::NoSuccessorOutput);
    }

    #[test]
    fn a_successor_paid_anywhere_else_reads_as_owned() {
        // The comparison that recognizing the successor by its program
        // would have made unfalsifiable: the same object, the same
        // amount, paid outside the covenant. Two terms move, because
        // the canonical flow is ownerless-lateral only while the object
        // is ownerless.
        use transaction::{TargetOutput, TargetTransaction};

        let sample = sample("minimum-two-ash-inputs");
        let decoded = TargetTransaction::decode(&sample.bytes).expect("the bytes decode");
        let elsewhere: Vec<u8> = std::iter::once(0x51)
            .chain(std::iter::once(0x20))
            .chain(std::iter::repeat_n(0xab, 32))
            .collect();
        assert_ne!(elsewhere, sample.program);
        let outputs: Vec<TargetOutput> = decoded
            .outputs()
            .iter()
            .map(|output| {
                TargetOutput::new(
                    output.asset(),
                    output.value(),
                    output.nonce(),
                    elsewhere.clone(),
                )
            })
            .collect();
        let moved = TargetTransaction::new(
            decoded.version(),
            decoded.inputs().to_vec(),
            outputs,
            decoded.lock_time(),
            decoded.witnesses().to_vec(),
        )
        .expect("the roles still form a transaction");

        let observed = read_accepted(
            &moved.encode(),
            &sample.coins,
            sample.asset,
            &sample.program,
            OPERATION,
        )
        .expect("the object is still findable by its asset");
        let differed = compare(&sample.expected, &observed);
        assert!(differed.contains(&ProjectionTerm::Ownership));
        assert!(differed.contains(&ProjectionTerm::Flow));
        assert!(
            !differed.contains(&ProjectionTerm::Successor),
            "the amount did not change and must not be reported as though it had"
        );
    }

    #[test]
    fn every_term_is_named_exactly_once() {
        // The census matches §17.4's list, and the names are distinct so
        // a report cannot merge two findings into one line.
        assert_eq!(ProjectionTerm::ALL.len(), 13);
        let names: std::collections::BTreeSet<&str> =
            ProjectionTerm::ALL.iter().map(|term| term.name()).collect();
        assert_eq!(names.len(), 13);
    }
}
