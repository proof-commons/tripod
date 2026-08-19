//! Census tests for the wire vocabulary.
//!
//! A spelling table is only usable as protocol data if it is total over
//! its owning census and injective in both directions. These tests state
//! that directly, so a primitive, capability, or evidence requirement
//! admitted upstream without a spelling fails here rather than in a
//! report that quietly omits it.

use std::collections::BTreeSet;

use target_elements::confidential::OpeningBlocker;
use target_elements::{ElementsCapability, OpcodeId, TargetEvidenceRequirementId};

use crate::vocabulary::{
    capability_from_name, capability_name, evidence_requirement_from_name,
    evidence_requirement_name, opcode_from_name, opcode_name, opening_blocker_from_name,
    opening_blocker_name,
};

#[test]
fn every_reviewed_primitive_has_one_spelling() {
    let mut spellings = BTreeSet::new();
    for id in OpcodeId::ALL {
        let name = opcode_name(*id).expect("every reviewed primitive is spelled");
        assert!(
            spellings.insert(name),
            "duplicate primitive spelling {name}"
        );
        assert_eq!(
            opcode_from_name(name),
            Some(*id),
            "the spelling {name} must name back its own primitive",
        );
    }
    assert_eq!(
        spellings.len(),
        OpcodeId::ALL.len(),
        "the primitive table states exactly the reviewed census",
    );
}

#[test]
fn every_capability_has_one_spelling() {
    let mut spellings = BTreeSet::new();
    for capability in ElementsCapability::ALL {
        let name = capability_name(*capability).expect("every capability is spelled");
        assert!(
            spellings.insert(name),
            "duplicate capability spelling {name}"
        );
        assert_eq!(
            capability_from_name(name),
            Some(*capability),
            "the spelling {name} must name back its own capability",
        );
    }
    assert_eq!(
        spellings.len(),
        ElementsCapability::ALL.len(),
        "the capability table states exactly the target census",
    );
}

#[test]
fn every_evidence_requirement_has_one_spelling() {
    let mut spellings = BTreeSet::new();
    for id in TargetEvidenceRequirementId::ALL {
        let name = evidence_requirement_name(*id).expect("every requirement is spelled");
        assert!(
            spellings.insert(name),
            "duplicate requirement spelling {name}"
        );
        assert_eq!(
            evidence_requirement_from_name(name),
            Some(*id),
            "the spelling {name} must name back its own requirement",
        );
    }
    assert_eq!(
        spellings.len(),
        TargetEvidenceRequirementId::ALL.len(),
        "the requirement table states exactly the target census",
    );
}

#[test]
fn every_opening_blocker_has_one_spelling() {
    let mut spellings = BTreeSet::new();
    for blocker in OpeningBlocker::ALL {
        let name = opening_blocker_name(*blocker).expect("every blocker is spelled");
        assert!(spellings.insert(name), "duplicate blocker spelling {name}");
        assert_eq!(
            opening_blocker_from_name(name),
            Some(*blocker),
            "the spelling {name} must name back its own blocker",
        );
    }
    assert_eq!(
        spellings.len(),
        OpeningBlocker::ALL.len(),
        "the blocker table states exactly the reviewed census",
    );
}

#[test]
fn an_unknown_spelling_names_nothing() {
    assert_eq!(opcode_from_name("not_a_reviewed_primitive"), None);
    assert_eq!(capability_from_name("not_a_capability"), None);
    assert_eq!(evidence_requirement_from_name("not_a_requirement"), None);
    assert_eq!(opening_blocker_from_name("not_a_blocker"), None);
}
