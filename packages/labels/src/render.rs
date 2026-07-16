use std::fmt::Write as _;

use crate::{owner::LabelOwner, registry::LabelRegistry, source::slash_path};

// Register tokens render as double-backtick spans: the label scanner
// treats those as nonparticipating examples, so a register can never
// mint or cite the labels it indexes (ADR-013 generated-register
// nonparticipation rule).

pub fn specification_register(registry: &LabelRegistry) -> String {
    let mut output = String::from(
        "# Specification Upstream Label Register\n\n> Generated from the specification LaTeX sources by `tripod-labels`.\n> Do not edit by hand.\n> The Attestation specification owns these labels.\n> Displayed tokens are nonparticipating examples.\n\n| Plan citation | Owner-local label | Source file |\n|---|---|---|\n",
    );
    for (label, mint) in registry.iter() {
        writeln!(
            output,
            "| ``[{}{}]`` | ``{label}`` | `{}` |",
            LabelOwner::Attestation.prefix(),
            label,
            slash_path(&mint.location.relative_path),
        )
        .expect("writing to a String cannot fail");
    }
    output
}

pub fn realization_register(registry: &LabelRegistry) -> String {
    let mut output = String::from(
        "# Realization Upstream Label Register\n\n> Generated from `docs/attestation/realization.md` by `tripod-labels`.\n> Do not edit by hand.\n> The realization document owns these labels.\n> Displayed tokens are nonparticipating examples.\n\n| Plan citation | Owner-local label | Owning heading |\n|---|---|---|\n",
    );
    for (label, mint) in registry.iter() {
        let home = mint
            .home
            .as_deref()
            .unwrap_or("<document>")
            .replace('|', "\\|");
        writeln!(
            output,
            "| ``[{}{}]`` | ``{label}`` | {home} |",
            LabelOwner::Realization.prefix(),
            label,
        )
        .expect("writing to a String cannot fail");
    }
    output
}

pub fn model_labels_json(registry: &LabelRegistry) -> Result<String, serde_json::Error> {
    Ok(serde_json::to_string_pretty(
        &registry
            .labels()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
    )? + "\n")
}
