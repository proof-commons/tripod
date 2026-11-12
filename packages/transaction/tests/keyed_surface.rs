//! Explicit-spelling locks, not Rust macro, alias or trait-implementation analysis.

fn public_fields(source: &str, name: &str) -> Option<Vec<String>> {
    let marker = format!("pub struct {name}");
    let body = source
        .split_once(&marker)?
        .1
        .split_once('{')?
        .1
        .split_once("\n}")?
        .0;
    Some(
        body.lines()
            .map(str::trim)
            .filter_map(|line| line.strip_prefix("pub "))
            .map(str::to_owned)
            .collect(),
    )
}

fn locked(source: &str) -> bool {
    public_fields(source, "KeyedOutputOffer").as_deref()
        == Some(&[
            "policy_asset: AssetId,".into(),
            "successor_value: u64,".into(),
            "additional_outputs: Vec<TargetOutput>,".into(),
            "fee: Option<u64>,".into(),
        ])
        && [
            "KeyedStructuralTransaction",
            "KeyedAdmittedTransaction",
            "KeyedPolicy",
        ]
        .iter()
        .all(|name| public_fields(source, name).is_some_and(|fields| fields.is_empty()))
}

#[test]
fn public_keyed_output_offer_has_no_script_or_stated_totals_bypass() {
    let source = include_str!("../src/keyed_construct.rs");
    assert!(locked(source));
    for replacement in ["pub script: Vec<u8>,", "pub stated_total: u64,"] {
        assert!(!locked(
            &source.replace("pub successor_value: u64,", replacement)
        ));
    }
    assert!(!locked(&source.replace(
        "    transaction: TargetTransaction,",
        "    pub transaction: TargetTransaction,"
    )));
    for name in ["construct_keyed", "admit_keyed"] {
        let signature = source
            .split_once(&format!("pub fn {name}"))
            .unwrap()
            .1
            .split_once('{')
            .unwrap()
            .0;
        assert!(!signature.contains("script:"));
        assert!(!signature.contains("total:"));
        assert!(!signature.contains("&mut"));
        assert!(signature.contains("KeyedConstructionRequest"));
        assert!(signature.contains("KeyedOutputOffer"));
    }
}

#[test]
fn public_construction_types_are_available_without_features() {
    use transaction::keyed_construct::{
        KeyedAdmittedTransaction, KeyedConstructionError, KeyedConstructionRequest,
        KeyedOutputOffer, KeyedPolicy, KeyedStructuralTransaction,
    };
    let sizes = [
        std::mem::size_of::<KeyedConstructionRequest<'_, '_>>(),
        std::mem::size_of::<KeyedOutputOffer>(),
        std::mem::size_of::<KeyedPolicy>(),
        std::mem::size_of::<KeyedStructuralTransaction<'_, '_>>(),
        std::mem::size_of::<KeyedAdmittedTransaction>(),
        std::mem::size_of::<KeyedConstructionError>(),
    ];
    assert!(sizes.into_iter().all(|size| size > 0));
}
