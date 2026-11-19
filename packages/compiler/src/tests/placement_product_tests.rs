//! Literal small-product controls for the checked placement representation.

use std::collections::BTreeSet;
use std::num::NonZeroU64;

use architecture::semantic::SemanticProfileId;

use crate::{
    carrier::CarrierQuantification,
    engine_identity::{self, CarrierKey, CaseKey, LayoutKey},
    engine_placement::{self as placement, Assignment, Candidate, EmptyCause, Entry, Error, Key},
    engine_records::{OperationKey, RelationKey},
    placement::{CarrierMultiplicity, PlacementSearchLimits, PlacementSearchReport},
};

fn key(code: u16) -> Key {
    Key {
        relation: RelationKey {
            operation: OperationKey {
                namespace: 1,
                code: 1,
            },
            kind: code,
            subject: vec![code],
        },
        case: CaseKey::new(
            SemanticProfileId::Attestation,
            vec![engine_identity::word(1_u16)],
        ),
    }
}

fn placed(code: u16, quantification: CarrierQuantification) -> placement::Placed {
    placement::Placed {
        carrier: CarrierKey::new(
            SemanticProfileId::Attestation,
            vec![engine_identity::word(code)],
        ),
        quantification,
    }
}

fn layout(code: u16) -> LayoutKey {
    LayoutKey::new(
        SemanticProfileId::Attestation,
        vec![engine_identity::word(code)],
    )
}

fn entry(code: u16, quantification: CarrierQuantification, layout_codes: &[u16]) -> Entry {
    Entry {
        placed: placed(code, quantification),
        layout: layout_codes.iter().map(|code| layout(*code)).collect(),
    }
}

fn row(
    code: u16,
    multiplicity: CarrierMultiplicity,
    entries: Vec<Entry>,
) -> placement::Eligibility {
    placement::Eligibility {
        key: key(code),
        multiplicity,
        options: entries.clone(),
        entries,
        empty_cause: EmptyCause::NoEligible,
    }
}

fn control() -> placement::Records {
    use CarrierQuantification::{CompleteFamilyProof, PerMember, Single};
    let eligibility = vec![
        row(
            1,
            CarrierMultiplicity::ExactlyOne,
            vec![entry(11, Single, &[91]), entry(12, Single, &[])],
        ),
        row(
            2,
            CarrierMultiplicity::AtLeastOne,
            vec![entry(21, Single, &[91]), entry(22, Single, &[92])],
        ),
        row(
            3,
            CarrierMultiplicity::EveryMember,
            vec![
                entry(31, PerMember, &[93]),
                entry(32, CompleteFamilyProof, &[93]),
            ],
        ),
        row(
            4,
            CarrierMultiplicity::DeliberateDuplication,
            vec![entry(41, Single, &[94]), entry(42, Single, &[])],
        ),
    ];
    placement::Records {
        plans: eligibility
            .iter()
            .map(|row| (row.key.clone(), true))
            .collect(),
        eligibility,
    }
}

fn limits(states: u64, candidates: u64) -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        NonZeroU64::new(states).expect("positive state budget"),
        NonZeroU64::new(candidates).expect("positive candidate budget"),
    )
}

fn expected_members() -> Vec<Candidate> {
    use CarrierQuantification::{CompleteFamilyProof, PerMember, Single};
    // This literal table and its loops form expectations without calling the
    // production option builder or reading factors back from the product.
    let first = [(11, Single, &[91_u16][..]), (12, Single, &[])];
    let second = [(21, Single, &[91_u16][..]), (22, Single, &[92_u16][..])];
    let third = [
        (31, PerMember, &[93_u16][..]),
        (32, CompleteFamilyProof, &[93_u16][..]),
    ];
    let fourth = [(41, Single, &[94_u16][..]), (42, Single, &[])];
    let mut expected = Vec::new();
    for a in first {
        for b in second {
            for c in third {
                let mut local_layout = BTreeSet::new();
                for code in a.2.iter().chain(b.2).chain(c.2).chain(fourth[0].2) {
                    local_layout.insert(layout(*code));
                }
                expected.push(Candidate {
                    assignments: vec![
                        Assignment {
                            key: key(1),
                            carriers: vec![placed(a.0, a.1)],
                        },
                        Assignment {
                            key: key(2),
                            carriers: vec![placed(b.0, b.1)],
                        },
                        Assignment {
                            key: key(3),
                            carriers: vec![placed(c.0, c.1)],
                        },
                        Assignment {
                            key: key(4),
                            carriers: fourth
                                .iter()
                                .map(|choice| placed(choice.0, choice.1))
                                .collect(),
                        },
                    ],
                    layout: local_layout.into_iter().collect(),
                });
            }
        }
    }
    expected.sort();
    expected
}

#[test]
fn placement_product_matches_exhaustive_oracle() {
    let records = control();
    let mut construction = PlacementSearchReport::default();
    let product = placement::product(&records, limits(100, 8), &mut construction)
        .expect("coherent literal factors");
    let expected = expected_members();
    assert_eq!(expected.len(), 8);
    assert!(expected.len() <= 4_096);
    assert_eq!(product.checked_cardinality(), Some(8));
    assert_eq!(
        product
            .factors()
            .map(|(_, options)| options.len())
            .collect::<Vec<_>>(),
        vec![2, 2, 2, 1]
    );
    assert_eq!(
        (construction.states_visited, construction.retained_options),
        (12, 7)
    );
    assert_eq!(
        (
            construction.complete_assignments,
            construction.feasible_placements
        ),
        (0, 0)
    );
    let mut expansion = PlacementSearchReport::default();
    let expanded = product
        .expand(limits(100, 8), &mut expansion)
        .expect("bounded expansion");
    let mut enumeration = PlacementSearchReport::default();
    let old = placement::enumerate(&records, limits(100, 8), &mut enumeration)
        .expect("old exhaustive witness");
    assert_eq!(expanded, expected);
    assert_eq!(old, expected);
    for member in &expected {
        assert_eq!(product.validate_member(member), Ok(()));
    }
    assert_eq!(
        product.expand(limits(100, 7), &mut PlacementSearchReport::default()),
        Err(Error::CandidateLimit(7))
    );

    let empty = placement::Records {
        plans: Vec::new(),
        eligibility: Vec::new(),
    };
    let zero = placement::product(&empty, limits(1, 1), &mut PlacementSearchReport::default())
        .expect("zero runtime factors");
    assert_eq!(zero.checked_cardinality(), Some(1));
    assert_eq!(
        zero.expand(limits(1, 1), &mut PlacementSearchReport::default()),
        Ok(vec![Candidate {
            assignments: Vec::new(),
            layout: Vec::new()
        }])
    );

    let mut quantification = expected[0].clone();
    quantification.assignments[2].carriers[0].quantification = CarrierQuantification::Single;
    assert!(matches!(
        product.validate_member(&quantification),
        Err(Error::UnpermittedCarrierPlacement { .. })
    ));
    let mut missing = expected[0].clone();
    missing
        .layout
        .retain(|requirement| *requirement != layout(91));
    assert!(matches!(
        product.validate_member(&missing),
        Err(Error::MissingLayoutRequirement(_))
    ));
    let mut surplus = expected[0].clone();
    surplus.layout.push(layout(99));
    assert_eq!(
        product.validate_member(&surplus),
        Err(Error::UnexpectedLayoutRequirement(vec![layout(99)]))
    );
}

#[test]
fn placement_product_rejects_incoherent_factors() {
    let records = control();
    let mut duplicated = records.clone();
    let repeated = duplicated.eligibility[0].options[0].clone();
    duplicated.eligibility[0].options.push(repeated);
    let normalized = placement::product(
        &duplicated,
        limits(100, 8),
        &mut PlacementSearchReport::default(),
    )
    .expect("duplicate local option normalizes");
    assert_eq!(
        normalized
            .factors()
            .next()
            .map(|(_, options)| options.len()),
        Some(2)
    );
    assert_eq!(normalized.checked_cardinality(), Some(8));

    // Another factor also supplies layout 91, so a whole-member union could
    // mask this changed local option. Product admission checks it locally.
    let mut incoherent = records.clone();
    incoherent.eligibility[0].options[0].layout.clear();
    assert_eq!(
        placement::product(
            &incoherent,
            limits(100, 8),
            &mut PlacementSearchReport::default()
        ),
        Err(Error::MissingLayoutRequirement(key(1)))
    );

    let mut missing = records;
    missing.eligibility.pop();
    assert!(matches!(
        placement::product(
            &missing,
            limits(100, 8),
            &mut PlacementSearchReport::default()
        ),
        Err(Error::PlacementCensusMismatch { .. })
    ));
}
