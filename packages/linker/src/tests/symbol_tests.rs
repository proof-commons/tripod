//! Pass one: the typed definition census (§14.3).

use std::num::NonZeroU32;

use tapscript::{BundleSymbol, CompactAshSymbols};

use crate::symbol::{SymbolType, collect_definitions, declared_type};
use crate::tests::{deployment, placeholder_symbols, relocatable_bundle, reviewed_target};
use crate::{DefinitionOrigin, LinkDeploymentParameters, LinkRefusal, SelfCommitmentStrategy};

#[test]
fn every_symbol_declares_a_type_and_every_definition_carries_it() {
    // The check pass one exists to make: a definition of the wrong kind
    // is refused even when it is the right width. Four of these
    // symbols resolve to thirty-two bytes under the reviewed contract,
    // so width alone could not separate them.
    let bundle = relocatable_bundle();
    let target = reviewed_target();
    let census = collect_definitions(
        &bundle,
        &deployment(
            &target,
            SelfCommitmentStrategy::ExternallyAuthenticatedCommitment,
        ),
    )
    .expect("the demonstration definitions collect");

    for (symbol, definition) in census.definitions() {
        assert_eq!(definition.symbol(), *symbol);
        assert_eq!(definition.value().symbol_type(), declared_type(*symbol));
    }
}

#[test]
fn the_declared_types_separate_the_thirty_two_byte_symbols() {
    // Written out, because this is the property the type check rests
    // on. If these four ever collapsed to one kind, a resolution
    // supplied under the wrong role would stop being detectable.
    assert_eq!(declared_type(BundleSymbol::ClosedAsset), SymbolType::Asset);
    assert_eq!(declared_type(BundleSymbol::ReserveAsset), SymbolType::Asset);
    assert_eq!(
        declared_type(BundleSymbol::TargetFeeRoleProgramDigest),
        SymbolType::ProgramDigest
    );
    assert_eq!(
        declared_type(BundleSymbol::UnspendableInternalKey),
        SymbolType::XOnlyPublicKey
    );
    assert_eq!(
        declared_type(BundleSymbol::AshConstructorProgram),
        SymbolType::WitnessProgram
    );
}

#[test]
fn the_census_splits_by_who_settles_each_symbol() {
    // Six resolved at link and fifteen defined by the bundle, and the
    // split must agree with the bundle's own symbol table rather than
    // with a list kept here. A symbol the deployment settled while the
    // table said the bundle would is ambiguous, not a preference. The
    // twenty-second symbol is settled by neither and appears in no
    // census at all, which is the ASH constructor's program.
    let bundle = relocatable_bundle();
    let target = reviewed_target();
    let census = collect_definitions(
        &bundle,
        &deployment(
            &target,
            SelfCommitmentStrategy::ExternallyAuthenticatedCommitment,
        ),
    )
    .expect("the demonstration definitions collect");

    assert_eq!(
        census
            .from_origin(DefinitionOrigin::DeploymentParameters)
            .count(),
        6
    );
    assert_eq!(census.from_origin(DefinitionOrigin::Bundle).count(), 15);
    assert_eq!(census.len(), 21);
    assert!(!census.is_empty());
}

#[test]
fn an_internal_key_of_the_wrong_width_never_reaches_the_census() {
    // The width check lives where the value is supplied, so a malformed
    // key is refused at the boundary rather than several stages later
    // with a confusing name.
    let target = reviewed_target();
    let refusal = LinkDeploymentParameters::new(
        &target,
        placeholder_symbols(&target),
        vec![0xa6; 31],
        SelfCommitmentStrategy::NotStated,
        NonZeroU32::new(8).expect("eight is nonzero"),
    )
    .expect_err("a thirty-one byte internal key is not an x-only public key");

    assert!(matches!(
        refusal,
        LinkRefusal::InvalidDeploymentParameters(_)
    ));
}

#[test]
fn a_symbol_of_the_wrong_width_never_reaches_the_link_at_all() {
    // The backend's own symbol type checks the widths of the five
    // program symbols, which is why the linker does not check them
    // again: there is one authored source for that question (§1.12),
    // and a resolution that is not the reviewed width cannot be built
    // into the value the link takes.
    let target = reviewed_target();
    assert!(
        CompactAshSymbols::new(
            &target,
            vec![0xa1; 31],
            vec![0xa2; 32],
            vec![0xa4; 32],
            0,
            vec![0xa5; 32],
        )
        .is_err()
    );
}
