//! The disclosure classes of Guide-13 §16.3, recorded separately.
//!
//! §16.3 names four classes and then states, for the private plan, what
//! six named items disclose. This module is that table as types, plus the
//! explicit plan's answer to the same six items, so a comparison between
//! the two representations is a difference between two values rather than
//! a paragraph somebody wrote after reading both.
//!
//! # Shape leakage is recorded as leakage
//!
//! §16.3's last row says input and output counts are public transaction
//! shape, and it is a *disclosure class* in the same list as the other
//! three. A private plan whose counts are public has leaked the counts;
//! it has not been excused them. So the counts appear here under
//! [`DisclosureClass::TransactionShapeLeakage`] with the same standing
//! under both plans, and [`shape_leakage`] is what a reader asks rather
//! than having to notice an absence.
//!
//! §16.5 is explicit that a successful private transfer establishes no
//! count privacy, and this table is where that shows up as a fact instead
//! of a caveat.
//!
//! # Every additional exact amount disclosure carries a typed reason
//!
//! §16.3's closing sentence. The explicit plan discloses both exact
//! receipt amount items that the private plan keeps private, and each of
//! those two disclosures is filed with an
//! [`AdditionalDisclosureReason`] naming the rule that requires it —
//! §6.2's locally checked aggregate arithmetic. A disclosure with no
//! reason is not representable.
//!
//! # Nothing here observes a transaction
//!
//! This module holds no bytes and reads no materialization. It is what
//! the two representation plans *state* about disclosure, which is the
//! thing a materialization can then be checked against
//! ([`crate::live_pairs`]).

use std::collections::{BTreeMap, BTreeSet};

use compiler::live_transfer_plan::LiveTransferRepresentationPlan;

/// One of §16.3's four disclosure classes.
///
/// Recorded separately, which is §16.3's own word for it: a single
/// "disclosure" column would let a semantic disclosure and a shape leak
/// cancel each other out in a summary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DisclosureClass {
    /// What the transfer means, published because it is the transfer.
    Semantic,
    /// What the target must see to decide safety.
    TargetSafety,
    /// What this deployment's own policy publishes.
    DeploymentPolicy,
    /// What the transaction's shape reveals whether or not anyone wanted
    /// it to.
    TransactionShapeLeakage,
}

impl DisclosureClass {
    /// All four, in §16.3's order.
    pub const ALL: &'static [Self] = &[
        Self::Semantic,
        Self::TargetSafety,
        Self::DeploymentPolicy,
        Self::TransactionShapeLeakage,
    ];

    /// The class's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Semantic => "semantic-disclosure",
            Self::TargetSafety => "target-safety-disclosure",
            Self::DeploymentPolicy => "deployment-policy-disclosure",
            Self::TransactionShapeLeakage => "transaction-shape-leakage",
        }
    }

    /// Whether this class is a leak rather than a publication.
    ///
    /// The distinction §16.5 turns on: the first three classes publish
    /// something because something requires it, and the fourth is what
    /// the shape gives away regardless.
    #[must_use]
    pub const fn is_leakage(self) -> bool {
        matches!(self, Self::TransactionShapeLeakage)
    }
}

/// One item §16.3 states a disclosure for.
///
/// The six rows of §16.3's private-plan table, in the guide's own order
/// and with its own names. A seventh item would be an item §16.3 did not
/// state, which is a change to the guide rather than to this list.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DisclosedItem {
    /// The exact amounts the consumed receipts carried.
    ExactInputReceiptAmounts,
    /// The exact amounts the created receipts carry.
    ExactOutputReceiptAmounts,
    /// The aggregate amount the transfer moves.
    AggregateSemanticTransferAmount,
    /// Who the receipts belong to.
    Owners,
    /// Which asset the receipts carry.
    AssetU,
    /// How many receipts go in and how many come out.
    InputAndOutputCounts,
}

impl DisclosedItem {
    /// All six, in §16.3's order.
    pub const ALL: &'static [Self] = &[
        Self::ExactInputReceiptAmounts,
        Self::ExactOutputReceiptAmounts,
        Self::AggregateSemanticTransferAmount,
        Self::Owners,
        Self::AssetU,
        Self::InputAndOutputCounts,
    ];

    /// The item's wire spelling, which is §16.3's own.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ExactInputReceiptAmounts => "exact-input-receipt-amounts",
            Self::ExactOutputReceiptAmounts => "exact-output-receipt-amounts",
            Self::AggregateSemanticTransferAmount => "aggregate-semantic-transfer-amount",
            Self::Owners => "owners",
            Self::AssetU => "asset-u",
            Self::InputAndOutputCounts => "input-and-output-counts",
        }
    }

    /// Whether this item is an exact receipt amount.
    ///
    /// The two items §16.3's closing sentence is about: an additional
    /// disclosure of either needs a typed reason, and no other item does.
    #[must_use]
    pub const fn is_an_exact_receipt_amount(self) -> bool {
        matches!(
            self,
            Self::ExactInputReceiptAmounts | Self::ExactOutputReceiptAmounts
        )
    }
}

/// What one item's disclosure standing is.
///
/// The right-hand column of §16.3's table, spelled exactly as the guide
/// spells it. `NotPublishedByProtocol` in particular is not `Private`:
/// §16.3 says the aggregate is *not published by the protocol*, which is
/// a statement about what this pipeline emits and not a claim that a
/// reader of the chain could never derive one.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DisclosureStanding {
    /// The value never appears in public protocol output.
    Private,
    /// The protocol publishes no such value.
    NotPublishedByProtocol,
    /// The value is committed by the constructor and is public in it.
    PublicConstructorMetadata,
    /// The value appears in the clear in a target field.
    Explicit,
    /// The value is readable from the transaction's shape.
    PublicTransactionShape,
}

impl DisclosureStanding {
    /// The standing's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::NotPublishedByProtocol => "not-published-by-protocol",
            Self::PublicConstructorMetadata => "public-constructor-metadata",
            Self::Explicit => "explicit",
            Self::PublicTransactionShape => "public-transaction-shape",
        }
    }

    /// Whether this standing puts an exact value in public output.
    #[must_use]
    pub const fn is_public(self) -> bool {
        matches!(
            self,
            Self::PublicConstructorMetadata | Self::Explicit | Self::PublicTransactionShape
        )
    }
}

/// Why a plan discloses an exact amount the other plan keeps private.
///
/// §16.3's closing sentence made a type. Each arm names the rule that
/// requires the disclosure, so a reader can check the requirement rather
/// than accept that one exists; there is deliberately no arm meaning "for
/// convenience" or "historically".
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum AdditionalDisclosureReason {
    /// §6.2 closes conservation in the target's own arithmetic.
    ///
    /// The explicit plan's coordinator computes the input and output sums
    /// and verifies equality inside the program, and a sum of fields the
    /// program cannot read is not a sum it can compute. So the amounts
    /// are explicit *because* the conservation check is local, and the
    /// private plan's amounts can be commitments precisely because §6.3
    /// moves that check to target CT conservation instead.
    ExplicitConservationIsCheckedInTheProgram,
}

impl AdditionalDisclosureReason {
    /// The reason's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ExplicitConservationIsCheckedInTheProgram => {
                "explicit-conservation-is-checked-in-the-program"
            }
        }
    }
}

/// One item's disclosure under one representation plan.
///
/// The reason travels with the row rather than in a table beside it: a
/// row whose standing said [`DisclosureStanding::Explicit`] for an exact
/// receipt amount and carried no reason is refused by
/// [`disclosure_table`]'s own invariant test, which is the only way
/// §16.3's closing sentence can be more than advice.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DisclosureRow {
    item: DisclosedItem,
    class: DisclosureClass,
    standing: DisclosureStanding,
    reason: Option<AdditionalDisclosureReason>,
}

impl DisclosureRow {
    /// The item this row is about.
    #[must_use]
    pub const fn item(&self) -> DisclosedItem {
        self.item
    }

    /// Which of §16.3's four classes it is recorded under.
    #[must_use]
    pub const fn class(&self) -> DisclosureClass {
        self.class
    }

    /// What the plan discloses about it.
    #[must_use]
    pub const fn standing(&self) -> DisclosureStanding {
        self.standing
    }

    /// The typed reason, where the row is an additional exact amount
    /// disclosure.
    #[must_use]
    pub const fn reason(&self) -> Option<AdditionalDisclosureReason> {
        self.reason
    }
}

/// One row with no additional disclosure to justify.
const fn row(
    item: DisclosedItem,
    class: DisclosureClass,
    standing: DisclosureStanding,
) -> DisclosureRow {
    DisclosureRow {
        item,
        class,
        standing,
        reason: None,
    }
}

/// One row that discloses an exact receipt amount, and why.
const fn disclosed(
    item: DisclosedItem,
    class: DisclosureClass,
    reason: AdditionalDisclosureReason,
) -> DisclosureRow {
    DisclosureRow {
        item,
        class,
        standing: DisclosureStanding::Explicit,
        reason: Some(reason),
    }
}

use AdditionalDisclosureReason as R;
use DisclosedItem as I;
use DisclosureClass as C;
use DisclosureStanding as D;

/// §16.3's private-plan table, exactly as the guide states it.
///
/// Six rows, six standings, and no editorializing. The classes are this
/// module's reading of which of §16.3's four each row belongs to, and the
/// reading is checkable: an owner is committed by the constructor the
/// deployment links, which is deployment policy; the asset is what the
/// target must see to settle conservation, which is target safety; the
/// aggregate is the transfer's own meaning; and the counts are shape.
pub const PRIVATE_PLAN_DISCLOSURE: &[DisclosureRow] = &[
    row(I::ExactInputReceiptAmounts, C::Semantic, D::Private),
    row(I::ExactOutputReceiptAmounts, C::Semantic, D::Private),
    row(
        I::AggregateSemanticTransferAmount,
        C::Semantic,
        D::NotPublishedByProtocol,
    ),
    row(I::Owners, C::DeploymentPolicy, D::PublicConstructorMetadata),
    row(I::AssetU, C::TargetSafety, D::Explicit),
    row(
        I::InputAndOutputCounts,
        C::TransactionShapeLeakage,
        D::PublicTransactionShape,
    ),
];

/// The explicit plan's answer to the same six items.
///
/// Four rows are identical to the private plan's and two are not, which
/// is the entire disclosure difference §13.3 asks a minimality row to
/// carry. The aggregate is `Explicit` here rather than
/// `NotPublishedByProtocol` for a reason worth stating: under §6.2 the
/// aggregate is not published as a field, but every summand is, and a
/// total anyone can add from public fields is not one this table may call
/// unpublished.
pub const EXPLICIT_PLAN_DISCLOSURE: &[DisclosureRow] = &[
    disclosed(
        I::ExactInputReceiptAmounts,
        C::TargetSafety,
        R::ExplicitConservationIsCheckedInTheProgram,
    ),
    disclosed(
        I::ExactOutputReceiptAmounts,
        C::TargetSafety,
        R::ExplicitConservationIsCheckedInTheProgram,
    ),
    disclosed(
        I::AggregateSemanticTransferAmount,
        C::Semantic,
        R::ExplicitConservationIsCheckedInTheProgram,
    ),
    row(I::Owners, C::DeploymentPolicy, D::PublicConstructorMetadata),
    row(I::AssetU, C::TargetSafety, D::Explicit),
    row(
        I::InputAndOutputCounts,
        C::TransactionShapeLeakage,
        D::PublicTransactionShape,
    ),
];

/// One representation plan's complete disclosure table.
#[must_use]
pub const fn disclosure_table(plan: LiveTransferRepresentationPlan) -> &'static [DisclosureRow] {
    match plan {
        LiveTransferRepresentationPlan::Explicit => EXPLICIT_PLAN_DISCLOSURE,
        LiveTransferRepresentationPlan::PrivateCommitted => PRIVATE_PLAN_DISCLOSURE,
    }
}

/// One plan's table, indexed by item.
#[must_use]
pub fn disclosure_index(
    plan: LiveTransferRepresentationPlan,
) -> BTreeMap<DisclosedItem, DisclosureRow> {
    disclosure_table(plan)
        .iter()
        .map(|entry| (entry.item, *entry))
        .collect()
}

/// Every item one plan leaks through the transaction's shape.
///
/// Recorded as leakage and never as an exemption. The set is the same
/// under both plans, which is exactly the finding: the private plan hides
/// the amounts and hides nothing about the counts.
#[must_use]
pub fn shape_leakage(plan: LiveTransferRepresentationPlan) -> BTreeSet<DisclosedItem> {
    disclosure_table(plan)
        .iter()
        .filter(|entry| entry.class.is_leakage())
        .map(|entry| entry.item)
        .collect()
}

/// Every additional exact amount disclosure one plan makes, with its
/// reason.
///
/// "Additional" is measured against the private plan, which is the
/// minimal one §13.3's question is asked about: an item the private plan
/// keeps private and this plan publishes is an additional disclosure, and
/// §16.3 requires each one to name a reason.
#[must_use]
#[expect(
    clippy::zero_sized_map_values,
    reason = "§16.3 requires each additional disclosure to name its own \
              reason, so the association is the value; the reason census \
              happens to have one member today and a set would drop the \
              association rather than represent it"
)]
pub fn additional_exact_amount_disclosures(
    plan: LiveTransferRepresentationPlan,
) -> BTreeMap<DisclosedItem, AdditionalDisclosureReason> {
    let private = disclosure_index(LiveTransferRepresentationPlan::PrivateCommitted);
    disclosure_table(plan)
        .iter()
        .filter(|entry| entry.item.is_an_exact_receipt_amount())
        .filter(|entry| {
            private
                .get(&entry.item)
                .is_none_or(|baseline| !baseline.standing.is_public())
        })
        .filter(|entry| entry.standing.is_public())
        .filter_map(|entry| entry.reason.map(|reason| (entry.item, reason)))
        .collect()
}

/// Every item the two plans disclose differently.
///
/// The disclosure comparison §13.5 recomputes, as a set rather than a
/// verdict: whether the difference *supports* minimality is §16.2's
/// question and this function deliberately does not answer it.
#[must_use]
pub fn disclosure_difference() -> BTreeMap<DisclosedItem, (DisclosureStanding, DisclosureStanding)>
{
    let explicit = disclosure_index(LiveTransferRepresentationPlan::Explicit);
    let private = disclosure_index(LiveTransferRepresentationPlan::PrivateCommitted);
    DisclosedItem::ALL
        .iter()
        .filter_map(|item| {
            let left = explicit.get(item)?.standing;
            let right = private.get(item)?.standing;
            (left != right).then_some((*item, (left, right)))
        })
        .collect()
}

/// Every class one plan records at least one row under.
#[must_use]
pub fn recorded_classes(
    plan: LiveTransferRepresentationPlan,
) -> BTreeMap<DisclosureClass, BTreeSet<DisclosedItem>> {
    let mut recorded: BTreeMap<DisclosureClass, BTreeSet<DisclosedItem>> = BTreeMap::new();
    for entry in disclosure_table(plan) {
        recorded.entry(entry.class).or_default().insert(entry.item);
    }
    recorded
}

#[cfg(test)]
mod tests {
    use super::{
        AdditionalDisclosureReason, DisclosedItem, DisclosureClass, DisclosureStanding,
        additional_exact_amount_disclosures, disclosure_difference, disclosure_index,
        disclosure_table, recorded_classes, shape_leakage,
    };
    use compiler::live_transfer_plan::LiveTransferRepresentationPlan;
    use std::collections::BTreeSet;

    /// Both admitted plans, which is the whole of §6.1.
    const PLANS: &[LiveTransferRepresentationPlan] = &[
        LiveTransferRepresentationPlan::Explicit,
        LiveTransferRepresentationPlan::PrivateCommitted,
    ];

    #[test]
    fn the_private_table_is_the_one_the_guide_states() {
        // §16.3's six rows, transcribed and checked line by line rather
        // than paraphrased. A row whose standing drifted would be this
        // package quietly restating the guide.
        let table = disclosure_index(LiveTransferRepresentationPlan::PrivateCommitted);
        assert_eq!(table.len(), 6);
        assert_eq!(
            table[&DisclosedItem::ExactInputReceiptAmounts].standing(),
            DisclosureStanding::Private,
        );
        assert_eq!(
            table[&DisclosedItem::ExactOutputReceiptAmounts].standing(),
            DisclosureStanding::Private,
        );
        assert_eq!(
            table[&DisclosedItem::AggregateSemanticTransferAmount].standing(),
            DisclosureStanding::NotPublishedByProtocol,
        );
        assert_eq!(
            table[&DisclosedItem::Owners].standing(),
            DisclosureStanding::PublicConstructorMetadata,
        );
        assert_eq!(
            table[&DisclosedItem::AssetU].standing(),
            DisclosureStanding::Explicit,
        );
        assert_eq!(
            table[&DisclosedItem::InputAndOutputCounts].standing(),
            DisclosureStanding::PublicTransactionShape,
        );
    }

    #[test]
    fn every_plan_states_every_item_exactly_once() {
        // A table missing an item would make a comparison silently
        // one-sided, and one stating an item twice would let two rows
        // disagree about the same disclosure.
        for plan in PLANS {
            let table = disclosure_table(*plan);
            let items: BTreeSet<_> = table.iter().map(super::DisclosureRow::item).collect();
            assert_eq!(items.len(), table.len(), "{plan:?} states an item twice");
            assert_eq!(
                items,
                DisclosedItem::ALL.iter().copied().collect::<BTreeSet<_>>(),
                "{plan:?} does not state every §16.3 item",
            );
        }
    }

    #[test]
    fn no_exact_amount_is_disclosed_without_a_typed_reason() {
        // §16.3's closing sentence, enforced. A row publishing an exact
        // receipt amount with no reason is the failure this check exists
        // for; a row keeping one private with a reason attached would be
        // a reason for nothing, and fails too.
        for plan in PLANS {
            for entry in disclosure_table(*plan) {
                if entry.item().is_an_exact_receipt_amount() && entry.standing().is_public() {
                    assert_ne!(
                        entry.reason(),
                        None,
                        "{plan:?} discloses {} with no reason",
                        entry.item().name(),
                    );
                }
                if entry.reason().is_some() {
                    assert!(
                        entry.standing().is_public(),
                        "{plan:?} gives a reason for disclosing nothing",
                    );
                }
            }
        }
    }

    #[test]
    fn the_explicit_plan_is_the_one_with_the_additional_disclosures() {
        // The disclosure comparison, as a difference between two values.
        // Both exact receipt amount items are additional under the
        // explicit plan and neither is under the private one, which is
        // the whole of what §13.3's question compares.
        let additional =
            additional_exact_amount_disclosures(LiveTransferRepresentationPlan::Explicit);
        assert_eq!(additional.len(), 2);
        for item in [
            DisclosedItem::ExactInputReceiptAmounts,
            DisclosedItem::ExactOutputReceiptAmounts,
        ] {
            assert_eq!(
                additional.get(&item).copied(),
                Some(AdditionalDisclosureReason::ExplicitConservationIsCheckedInTheProgram),
            );
        }
        assert_eq!(
            additional_exact_amount_disclosures(LiveTransferRepresentationPlan::PrivateCommitted)
                .len(),
            0,
        );
    }

    #[test]
    fn the_counts_leak_under_both_plans_and_are_recorded_as_leakage() {
        // §16.5 denies count privacy, and this is where that stops being
        // a caveat: the shape leakage is the same set under both plans,
        // so the private plan hides nothing about the counts.
        let expected = BTreeSet::from([DisclosedItem::InputAndOutputCounts]);
        for plan in PLANS {
            assert_eq!(shape_leakage(*plan), expected, "{plan:?} leaks another set");
            let classes = recorded_classes(*plan);
            assert_eq!(
                classes[&DisclosureClass::TransactionShapeLeakage],
                expected,
                "{plan:?} files the counts somewhere other than leakage",
            );
        }
        assert!(DisclosureClass::TransactionShapeLeakage.is_leakage());
        for class in DisclosureClass::ALL {
            if *class != DisclosureClass::TransactionShapeLeakage {
                assert!(!class.is_leakage());
            }
        }
    }

    #[test]
    fn the_two_plans_differ_in_exactly_the_amount_items() {
        // The comparison the report carries: three items differ and
        // three agree, and every one of the three that differ is an
        // amount. A difference in the owners or in the asset would be a
        // representation difference §6.3 does not admit.
        let difference = disclosure_difference();
        assert_eq!(
            difference.keys().copied().collect::<BTreeSet<_>>(),
            BTreeSet::from([
                DisclosedItem::ExactInputReceiptAmounts,
                DisclosedItem::ExactOutputReceiptAmounts,
                DisclosedItem::AggregateSemanticTransferAmount,
            ]),
        );
        for (item, (explicit, private)) in &difference {
            assert!(explicit.is_public(), "{} is not public", item.name());
            assert!(!private.is_public(), "{} is public", item.name());
        }
        assert_eq!(DisclosureClass::ALL.len(), 4);
    }
}
