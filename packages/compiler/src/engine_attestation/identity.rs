//! Attestation-owned structural identity encoders.
#[cfg(test)]
use crate::engine_identity::SymbolKey;
use crate::engine_identity::{
    CarrierKey, CaseKey, LayoutKey, Part, PurposeKey, sequence, side_code, word,
};
use architecture::semantic::SemanticProfileId;
fn relation(value: &realization::RelationId) -> Part {
    let value = crate::engine_attestation::relation_key(value);
    sequence([
        word(value.operation.namespace),
        word(value.operation.code),
        word(value.kind),
        sequence(value.subject.into_iter().map(word)),
    ])
}

pub fn attestation_case(value: &crate::case::ExecutionCaseId) -> CaseKey {
    CaseKey::new(
        SemanticProfileId::Attestation,
        attestation_case_parts(value),
    )
}
fn attestation_case_parts(value: &crate::case::ExecutionCaseId) -> Vec<Part> {
    vec![
        word(value.operation.code()),
        word(value.sponsor as u16),
        sequence(
            value
                .representations
                .iter()
                .map(|(object, mode)| sequence([word(object.code()), word(*mode as u16)])),
        ),
    ]
}
pub fn attestation_carrier(value: &crate::carrier::CarrierRole) -> CarrierKey {
    CarrierKey::new(
        SemanticProfileId::Attestation,
        attestation_carrier_parts(value),
    )
}
fn attestation_carrier_parts(value: &crate::carrier::CarrierRole) -> Vec<Part> {
    use crate::carrier::CarrierRole as C;
    match value {
        C::EveryInputFamilyMember { object } => vec![word(0u16), word(object.code())],
        C::InputFamilyCoordinator { object } => vec![word(1u16), word(object.code())],
        C::OperationGlobal { operation, anchor } => {
            vec![word(2u16), word(operation.code()), word(anchor.code())]
        }
        C::BackendStructural { operation } => vec![word(3u16), word(operation.code())],
        C::ExternalEvidence { requirement } => vec![word(4u16), evidence(requirement)],
    }
}
fn evidence(value: &realization::ExternalEvidenceRequirement) -> Part {
    use realization::ExternalEvidenceRequirement as E;
    sequence(match value {
        E::OperatorAuthorization { operation } => vec![word(0u16), word(operation.code())],
        E::ConfidentialValueConservation { operation, asset } => {
            vec![word(1u16), word(operation.code()), word(asset.code())]
        }
        E::SubstrateConservation { operation, asset } => {
            vec![word(2u16), word(operation.code()), word(asset.code())]
        }
    })
}
fn source(value: &crate::source::SourceRequirement) -> Part {
    use crate::source::{OperandRole as O, RequirementActivation as A};
    let role = match value.operand.role() {
        O::ObjectFamilyMembers { side, object } => {
            vec![word(0u16), word(side_code(*side)), word(object.code())]
        }
        O::ObjectFamilyCount { side, object } => {
            vec![word(1u16), word(side_code(*side)), word(object.code())]
        }
        O::ObjectFamilyAmount { side, object } => {
            vec![word(2u16), word(side_code(*side)), word(object.code())]
        }
        O::ObjectFamilyOwners { object } => vec![word(3u16), word(object.code())],
        O::ProtocolSignerSet => vec![word(4u16)],
        O::SponsorSignerSet => vec![word(5u16)],
        O::SponsorRegion => vec![word(6u16)],
        O::CanonicalPartition => vec![word(7u16)],
        O::OpenFlowSet => vec![word(8u16)],
        O::RootEffects => vec![word(9u16)],
        O::ProjectionSet => vec![word(10u16)],
        O::RuntimeBound { bound } => vec![word(11u16), word(bound.code())],
        O::Representation { object } => vec![word(12u16), word(object.code())],
        O::ConstructibilityCase => vec![word(13u16)],
        O::LifecycleGraph => vec![word(14u16)],
        O::ExpressionResult { expression } => vec![
            word(15u16),
            sequence(match expression {
                realization::ExprId::Fact(fact) => vec![
                    word(0u16),
                    sequence(
                        crate::engine_attestation::constructibility::fact_key(fact)
                            .into_iter()
                            .map(word),
                    ),
                ],
                realization::ExprId::Relation { relation: id, role } => {
                    vec![word(1u16), relation(id), word(*role as u16)]
                }
            }),
        ],
        O::ExternalEvidence { requirement } => vec![word(16u16), evidence(requirement)],
    };
    let availability =
        crate::engine_attestation::constructibility::availability_key(value.availability);
    let activation = match value.activation {
        A::Always => vec![word(0u16)],
        A::WhenSponsorPresent => vec![word(1u16)],
        A::WhenRepresentation { object, mode } => {
            vec![word(2u16), word(object.code()), word(mode as u16)]
        }
    };
    sequence([
        relation(value.operand.relation()),
        sequence(role),
        word(value.source as u16),
        sequence([word(availability.kind), word(availability.object)]),
        sequence(activation),
    ])
}
pub fn attestation_layout(value: &crate::layout::LayoutRequirement) -> LayoutKey {
    LayoutKey::new(
        SemanticProfileId::Attestation,
        attestation_layout_parts(value),
    )
}
fn attestation_layout_parts(value: &crate::layout::LayoutRequirement) -> Vec<Part> {
    use crate::layout::LayoutRequirement as L;
    match value {
        L::AuthenticateFamilyCensus {
            relation: id,
            side,
            object,
        } => vec![
            word(0u16),
            relation(id),
            word(side_code(*side)),
            word(object.code()),
        ],
        L::CompleteAndDisjointFamilies { relation: id, side } => {
            vec![word(1u16), relation(id), word(side_code(*side))]
        }
        L::CanonicalCoordinator { operation, anchor } => vec![
            word(2u16),
            word(operation.code()),
            CarrierKey::part(
                SemanticProfileId::Attestation,
                attestation_carrier_parts(anchor),
            ),
        ],
        L::MakeSourceAvailable {
            relation: id,
            case,
            carrier,
            source: row,
        } => vec![
            word(3u16),
            relation(id),
            CaseKey::part(SemanticProfileId::Attestation, attestation_case_parts(case)),
            CarrierKey::part(
                SemanticProfileId::Attestation,
                attestation_carrier_parts(carrier),
            ),
            source(row),
        ],
        L::IsolateSponsorRegion { relation: id, case } => {
            vec![
                word(4u16),
                relation(id),
                CaseKey::part(SemanticProfileId::Attestation, attestation_case_parts(case)),
            ]
        }
        L::EnforceRepresentation {
            relation: id,
            object,
            representation,
        } => vec![
            word(5u16),
            relation(id),
            word(object.code()),
            word(*representation as u16),
        ],
        L::SecretFreeOperationPath { relation: id, case } => {
            vec![
                word(6u16),
                relation(id),
                CaseKey::part(SemanticProfileId::Attestation, attestation_case_parts(case)),
            ]
        }
    }
}
fn mutation(value: crate::coverage::RelationMutation) -> Part {
    use crate::coverage::{CardinalityCeiling as C, RelationMutation as M};
    let (tag, payload) = match value {
        M::CardinalityBelowMinimum => (0u16, vec![]),
        M::CardinalityAboveMaximum { ceiling } => (
            1,
            match ceiling {
                C::Declared(count) => vec![word(0u16), word(count.get())],
                C::Bound(bound) => vec![word(1u16), word(bound.code())],
            },
        ),
        M::UndeclaredObjectFamily => (2, vec![]),
        M::WrongRecognizedAsset => (3, vec![]),
        M::WrongRecognizedObject => (4, vec![]),
        M::AmountMismatch => (5, vec![]),
        M::MissingRequiredOwner => (6, vec![]),
        M::UnexpectedProtocolSecret => (7, vec![]),
        M::SponsorProtocolOverlap => (8, vec![]),
        M::SponsorMemberUnclaimed => (9, vec![]),
        M::MissingSponsorAuthorization => (10, vec![]),
        M::SponsorEnvelopeMultiplicityExceeded => (11, vec![]),
        M::WrongRootEffect => (12, vec![]),
        M::MissingRequiredProjection => (13, vec![]),
        M::ForbiddenProjectionPresent => (14, vec![]),
        M::MissingCanonicalDeltaFamily => (15, vec![]),
        M::UnexpectedCanonicalDeltaFamily => (16, vec![]),
        M::DuplicateCanonicalSourceOrDestination => (17, vec![]),
        M::UndeclaredOpenFlow => (18, vec![]),
        M::ConstructibilityWitnessUnavailable => (19, vec![]),
        M::PermissionlessPrivateDependency => (20, vec![]),
        M::UnsupportedRepresentation => (21, vec![]),
        M::UnauthenticatedRepresentation => (22, vec![]),
        M::RequiredLifecycleExitMissing => (23, vec![]),
        M::ExpressionPredicateFalse => (24, vec![]),
        M::ExternalEvidenceMissing => (25, vec![]),
        M::ExternalEvidenceFailed => (26, vec![]),
        M::ExternalEvidenceIdentityMismatch => (27, vec![]),
    };
    sequence([word(tag), sequence(payload)])
}
fn placed(value: &crate::placement::PlacedCarrier) -> Part {
    sequence([
        CarrierKey::part(
            SemanticProfileId::Attestation,
            attestation_carrier_parts(&value.carrier),
        ),
        word(value.quantification as u16),
    ])
}
pub fn attestation_purpose(value: &crate::coverage::CoveragePurpose) -> PurposeKey {
    PurposeKey::new(
        SemanticProfileId::Attestation,
        attestation_purpose_parts(value),
    )
}
fn attestation_purpose_parts(value: &crate::coverage::CoveragePurpose) -> Vec<Part> {
    use crate::coverage::CoveragePurpose as P;
    match value {
        P::ActiveAccept => vec![word(0u16)],
        P::InactiveAccept => vec![word(1u16)],
        P::FocusedReject(value) => vec![word(2u16), mutation(*value)],
        P::CarrierExecution(value) => vec![word(3u16), placed(value)],
        P::AcceptedProjection => vec![word(4u16)],
        P::StructuralPresence => vec![word(5u16)],
        P::StructuralRejection(value) => vec![word(6u16), mutation(*value)],
        P::ExternalEvidenceVerified { requirement } => vec![word(7u16), evidence(requirement)],
        P::ExternalEvidenceRejected {
            requirement,
            mutation: value,
        } => vec![word(8u16), evidence(requirement), mutation(*value)],
    }
}
#[cfg(test)]
pub fn attestation_symbol(value: &crate::coverage_graph::CoverageNodeId) -> SymbolKey {
    SymbolKey::new(
        SemanticProfileId::Attestation,
        attestation_symbol_parts(value),
    )
}
#[cfg(test)]
fn attestation_symbol_parts(value: &crate::coverage_graph::CoverageNodeId) -> Vec<Part> {
    use crate::coverage_graph::CoverageNodeId as N;
    let case = |value: &crate::placement::RelationCaseKey| {
        sequence([
            relation(&value.relation),
            CaseKey::part(
                SemanticProfileId::Attestation,
                attestation_case_parts(&value.case),
            ),
        ])
    };
    match value {
        N::RelationCase(value) => vec![word(0u16), case(value)],
        N::Requirement(value) => vec![
            word(1u16),
            relation(&value.relation),
            CaseKey::part(
                SemanticProfileId::Attestation,
                attestation_case_parts(&value.case),
            ),
            word(value.boundary as u16),
            PurposeKey::part(
                SemanticProfileId::Attestation,
                attestation_purpose_parts(&value.purpose),
            ),
        ],
        N::Carrier {
            relation_case,
            carrier,
        } => vec![word(2u16), case(relation_case), placed(carrier)],
        N::Layout(value) => vec![
            word(3u16),
            LayoutKey::part(
                SemanticProfileId::Attestation,
                attestation_layout_parts(value),
            ),
        ],
        N::ExternalEvidence(value) => vec![word(4u16), evidence(value)],
    }
}

pub fn authorization(codes: &[u16]) -> crate::engine_identity::AuthorizationKey {
    crate::engine_identity::AuthorizationKey::new(
        SemanticProfileId::Attestation,
        codes.iter().copied().map(word).collect(),
    )
}
