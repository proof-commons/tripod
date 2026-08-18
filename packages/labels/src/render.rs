use std::fmt::Write as _;

use crate::{
    attestation::{AttestationBase, Source},
    owner::LabelOwner,
    registry::LabelRegistry,
    source::slash_path,
};

// Register tokens render as double-backtick spans: the label scanner
// treats those as nonparticipating examples, so a register can never
// mint or cite the labels it indexes (ADR-019 generated-register
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

/// Render the companion attestation register (ADR-020).
///
/// Three views of one evidence base: the first-hand rows with their
/// evidence, the status map, and the homonymy of the effective
/// relation. None of the three creates what it presents.
pub fn attestation_register(base: &AttestationBase) -> String {
    let mut output = String::from(
        "# Companion Attestation Register\n\n> Generated from the archived kind registry and its adopting record by `tripod-labels`.\n> Do not edit by hand.\n> This repository is the acceptee; the register views its evidence base and status map.\n> Displayed tokens are nonparticipating examples.\n\nRecords are totally ordered by name, kind, source, locator, then the\nsequence of the record in its source table.\n\n## Evidence\n\nThe recorded extension rows, held first-hand: a quoted spelling in this\ncorpus, the locator that places it, the catalogued sense, and this\ntree's mint census of the kind.\n\n| Name | Kind | Sense | Spelling | Locator | Mints |\n|---|---|---|---|---|---|\n",
    );
    for record in base.extensions() {
        writeln!(
            output,
            "| {} | ``{}`` | {} | ``{}`` | `{}` | {} |",
            cell(&record.key.name),
            record.key.kind,
            cell(record.sense.as_deref().unwrap_or_default()),
            record.spelling.as_deref().unwrap_or_default(),
            record.key.locator,
            base.mints(&record.key.kind),
        )
        .expect("writing to a String cannot fail");
    }

    write!(
        output,
        "\n## Statuses\n\nEvery pair of the effective relation carries exactly one admitting\nstatus. The {} base rows hold the edition's status by reference; below\nare the ones the edition does not record as firm, together with the\nrecorded extensions, which are firm on the evidence above. The\nedition's candidate is no member of the relation and appears nowhere\nhere.\n\n| Name | Kind | Source | Status | Held |\n|---|---|---|---|---|\n",
        base.base_count(),
    )
    .expect("writing to a String cannot fail");
    for record in base.borderline_base().chain(base.extensions()) {
        writeln!(
            output,
            "| {} | ``{}`` | {} | {} | {} |",
            cell(&record.key.name),
            record.key.kind,
            record.key.source.token(),
            record.status.token(),
            match record.key.source {
                Source::Base => "by reference",
                Source::Extension => "first-hand",
            },
        )
        .expect("writing to a String cannot fail");
    }

    let homonyms = base.homonyms();
    let names = homonyms
        .iter()
        .map(|record| record.key.name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    write!(
        output,
        "\n## Homonyms\n\nEvery pair of the effective relation whose name carries another kind:\n\
         {} pairs over {} names. A name here needs the kind token at its\nlabel to fix the \
         catalogued sense. Derived from the rows above,\ndeclared nowhere.\n\n\
         | Name | Kind | Source |\n|---|---|---|\n",
        homonyms.len(),
        names.len(),
    )
    .expect("writing to a String cannot fail");
    for record in homonyms {
        writeln!(
            output,
            "| {} | ``{}`` | {} |",
            cell(&record.key.name),
            record.key.kind,
            record.key.source.token(),
        )
        .expect("writing to a String cannot fail");
    }
    output
}

/// Prose copied from a source document into a table cell.
///
/// Pipes are escaped so the cell cannot split the row, and every
/// backtick run is widened so a copied token is displayed rather than
/// meant: a register participates in nothing it indexes.
fn cell(value: &str) -> String {
    value.replace('|', "\\|").replace('`', "``")
}

pub fn model_labels_json(registry: &LabelRegistry) -> Result<String, serde_json::Error> {
    Ok(serde_json::to_string_pretty(
        &registry
            .labels()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
    )? + "\n")
}
