use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use architecture::{InvariantClauseId, WitnessId};
use petgraph::{
    Direction,
    algo::is_cyclic_directed,
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};
use thiserror::Error;

use crate::{
    census::RepositoryCensus,
    diagnostic::{LabelDiagnostic, LabelErrorCode, sort_diagnostics},
    label::{Label, LabelShape},
    latex::harvest_attestation,
    markdown::{InlineCodeContext, fence_close, fence_open, scan_markdown},
    owner::{ImportedLabel, LabelOwner},
    registry::{LabelMint, LabelRegistry, RegistrySet},
    render,
    rust_source::{RustHarvest, harvest_crates, harvest_model},
    source::{SourceLocation, relative_to},
};

/// Class of one citation occurrence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CitationClass {
    AuthoredSameOwner,
    AuthoredImported,
    SyntheticArchitecture,
}

/// Stable origin of one citation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CitationOrigin {
    Source {
        owner: LabelOwner,
        location: SourceLocation,
    },
    ArchitectureWitness(WitnessId),
    ArchitectureClause(InvariantClauseId),
}

/// One citation occurrence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelCitation {
    pub source_owner: LabelOwner,
    pub target: ImportedLabel,
    pub origin: CitationOrigin,
    pub class: CitationClass,
}

/// Complete stable identity of one label-graph node.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LabelGraphNodeId {
    Mint {
        owner: LabelOwner,
        label: Label,
    },
    Citation {
        origin: CitationOrigin,
        target_owner: LabelOwner,
        label: Label,
    },
}

/// Direct Petgraph node weight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LabelGraphNode {
    Mint(LabelMint),
    Citation(LabelCitation),
}

impl LabelGraphNode {
    #[must_use]
    pub fn id(&self) -> LabelGraphNodeId {
        match self {
            Self::Mint(mint) => LabelGraphNodeId::Mint {
                owner: mint.owner.clone(),
                label: mint.label.clone(),
            },
            Self::Citation(citation) => LabelGraphNodeId::Citation {
                origin: citation.origin.clone(),
                target_owner: citation.target.owner.clone(),
                label: citation.target.label.clone(),
            },
        }
    }
}

/// Direct Petgraph citation-resolution edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LabelGraphEdge {
    ResolvesTo,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LabelGraphEdgeProjection {
    pub source: LabelGraphNodeId,
    pub target: LabelGraphNodeId,
    pub edge: LabelGraphEdge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelGraphProjection {
    pub nodes: Vec<LabelGraphNodeId>,
    pub edges: Vec<LabelGraphEdgeProjection>,
}

#[derive(Default)]
pub struct RepositoryLabels {
    pub registries: RegistrySet,
    pub diagnostics: Vec<LabelDiagnostic>,
    pub graph: DiGraph<LabelGraphNode, LabelGraphEdge, u32>,
    pub graph_node_by_id: BTreeMap<LabelGraphNodeId, NodeIndex<u32>>,
    pending_citations: Vec<LabelCitation>,
    attestation_anchor_names: Vec<String>,
    attestation_index_names: BTreeSet<String>,
    attestation_index_location: Option<SourceLocation>,
}
impl RepositoryLabels {
    pub fn harvest_sources(paths: &RepositoryCensus) -> Self {
        let mut result = Self::default();
        let (attestation, diagnostics) = harvest_attestation(paths);
        result.registries.attestation = attestation;
        result.diagnostics.extend(diagnostics);
        harvest_realization(paths, &mut result);
        harvest_adrs(paths, &mut result);
        harvest_plans(paths, &mut result);
        harvest_docs(paths, &mut result);
        let model = harvest_model(paths);
        add_model(model, &mut result);
        for (name, harvest) in harvest_crates(paths) {
            result.pending_citations.extend(harvest.citations);
            result.diagnostics.extend(harvest.diagnostics);
            result.registries.crates.insert(name, harvest.registry);
        }
        validate(&mut result);
        sort_diagnostics(&mut result.diagnostics);
        result
    }
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(LabelDiagnostic::is_error)
    }
    pub fn imported_citation_count(&self) -> usize {
        self.graph
            .node_weights()
            .filter(|node| {
                matches!(
                    node,
                    LabelGraphNode::Citation(LabelCitation {
                        class: CitationClass::AuthoredImported,
                        ..
                    })
                )
            })
            .count()
    }

    fn push_same_owner_citation(
        &mut self,
        owner: LabelOwner,
        label: Label,
        location: SourceLocation,
    ) {
        self.pending_citations.push(LabelCitation {
            source_owner: owner.clone(),
            target: ImportedLabel {
                owner: owner.clone(),
                label,
            },
            origin: CitationOrigin::Source { owner, location },
            class: CitationClass::AuthoredSameOwner,
        });
    }

    fn push_imported_citation(
        &mut self,
        source_owner: LabelOwner,
        target: ImportedLabel,
        location: SourceLocation,
    ) {
        self.pending_citations.push(LabelCitation {
            source_owner: source_owner.clone(),
            target,
            origin: CitationOrigin::Source {
                owner: source_owner,
                location,
            },
            class: CitationClass::AuthoredImported,
        });
    }
}

fn add_model(model: RustHarvest, result: &mut RepositoryLabels) {
    result.pending_citations.extend(model.citations);
    result.registries.model = model.registry;
    result.diagnostics.extend(model.diagnostics);
}
fn harvest_realization(paths: &RepositoryCensus, result: &mut RepositoryLabels) {
    let relative = relative_to(&paths.root, &paths.realization);
    let Ok(source) = fs::read_to_string(&paths.realization) else {
        result.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::Io,
            &SourceLocation::new(relative, 1, 1),
            "cannot read the realization",
        ));
        return;
    };
    harvest_attestation_citations(&relative, &source, result);
    let scan = scan_markdown(&relative, &source);
    result.diagnostics.extend(scan.diagnostics);
    for span in scan.code_spans {
        if span.delimiter_len != 1 {
            continue;
        }
        if let Some(token) = square(&span.content) {
            if !token.starts_with("A-") {
                if let Ok(imported) = ImportedLabel::parse(token) {
                    result.push_imported_citation(LabelOwner::Realization, imported, span.location);
                } else if let Ok(label) = Label::parse(token, LabelShape::Model) {
                    result.push_imported_citation(
                        LabelOwner::Realization,
                        ImportedLabel {
                            owner: LabelOwner::Model,
                            label,
                        },
                        span.location,
                    );
                }
            }
            continue;
        }
        let Ok(label) = Label::parse(span.content.trim(), LabelShape::Realization) else {
            continue;
        };
        match span.context {
            InlineCodeContext::Bare => {
                let mint = LabelMint {
                    owner: LabelOwner::Realization,
                    label,
                    location: span.location.clone(),
                    home: span.home,
                };
                result.registries.realization.insert_or_diagnose(
                    mint,
                    "the realization",
                    &mut result.diagnostics,
                );
            }
            InlineCodeContext::Parenthesized => {
                result.push_same_owner_citation(LabelOwner::Realization, label, span.location);
            }
            InlineCodeContext::Asymmetric => result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::AsymmetricCitation,
                &span.location,
                "label citation has an unmatched parenthesis",
            )),
        }
    }
}
fn harvest_adrs(paths: &RepositoryCensus, result: &mut RepositoryLabels) {
    for path in &paths.adrs {
        let Some(number) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.get(..3))
            .and_then(|number| number.parse::<u16>().ok())
        else {
            continue;
        };
        let relative = relative_to(&paths.root, path);
        let Ok(source) = fs::read_to_string(path) else {
            continue;
        };
        let scan = scan_markdown(&relative, &source);
        result.diagnostics.extend(scan.diagnostics);
        let mut registry = LabelRegistry::default();
        for span in scan.code_spans {
            if span.delimiter_len != 1 {
                continue;
            }
            if let Some(token) = square(&span.content) {
                if span.context == InlineCodeContext::Parenthesized {
                    import(token, &span.location, LabelOwner::Adr(number), result);
                } else {
                    result.diagnostics.push(LabelDiagnostic::error(
                        LabelErrorCode::InvalidImportedCitationForm,
                        &span.location,
                        "imported citation must be parenthesized",
                    ));
                }
                continue;
            }
            let Ok(label) = Label::parse(span.content.trim(), LabelShape::Adr) else {
                continue;
            };
            match span.context {
                InlineCodeContext::Bare => {
                    let mint = LabelMint {
                        owner: LabelOwner::Adr(number),
                        label,
                        location: span.location.clone(),
                        home: span.home,
                    };
                    registry.insert_or_diagnose(
                        mint,
                        &format!("ADR{number:03}"),
                        &mut result.diagnostics,
                    );
                }
                InlineCodeContext::Parenthesized => {
                    result.push_same_owner_citation(LabelOwner::Adr(number), label, span.location);
                }
                InlineCodeContext::Asymmetric => result.diagnostics.push(LabelDiagnostic::error(
                    LabelErrorCode::AsymmetricCitation,
                    &span.location,
                    "label citation has an unmatched parenthesis",
                )),
            }
        }
        result.registries.adrs.insert(number, registry);
    }
}
// Planning and repository-documentation Markdown are complete label
// owners (ADR-013): a bare planning-shaped label mints, a parenthesized
// one cites, and citations resolve against the complete owner registry
// across files — not merely against imports.

/// The two Markdown owners this harvest serves. A dedicated enum keeps
/// the registry and citation-sink dispatch exhaustive: a future owner
/// must choose destinations explicitly instead of falling into a
/// catch-all.
#[derive(Clone, Copy)]
enum MarkdownOwner {
    Plan,
    Doc,
}

impl MarkdownOwner {
    const fn owner(self) -> LabelOwner {
        match self {
            Self::Plan => LabelOwner::Plan,
            Self::Doc => LabelOwner::Doc,
        }
    }
    const fn name(self) -> &'static str {
        match self {
            Self::Plan => "PLAN",
            Self::Doc => "DOC",
        }
    }
}

fn harvest_plans(paths: &RepositoryCensus, result: &mut RepositoryLabels) {
    for path in &paths.plans {
        if *path == paths.specification_register || *path == paths.realization_register {
            // Generated registers are derivative publications and must
            // not contribute source mints or citations.
            continue;
        }
        harvest_markdown_owner(paths, path, MarkdownOwner::Plan, result);
    }
}

/// Harvest every authored Markdown file outside the trees owned
/// elsewhere as the `DOC` owner. Membership comes from the argument
/// census; the census verifier keeps that census welded to the walked
/// reality (ADR-014), so a newly added README still cannot silently
/// sit outside the label graph.
fn harvest_docs(paths: &RepositoryCensus, result: &mut RepositoryLabels) {
    for path in &paths.docs {
        harvest_markdown_owner(paths, path, MarkdownOwner::Doc, result);
    }
}

fn harvest_markdown_owner(
    paths: &RepositoryCensus,
    path: &Path,
    owner: MarkdownOwner,
    result: &mut RepositoryLabels,
) {
    let relative = relative_to(&paths.root, path);
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::Io,
                &SourceLocation::new(relative, 1, 1),
                error.to_string(),
            ));
            return;
        }
    };
    let scan = scan_markdown(&relative, &source);
    result.diagnostics.extend(scan.diagnostics);
    for span in scan.code_spans {
        if span.delimiter_len != 1 {
            continue;
        }
        if let Some(token) = square(&span.content) {
            if span.context == InlineCodeContext::Parenthesized {
                import(token, &span.location, owner.owner(), result);
            } else {
                result.diagnostics.push(LabelDiagnostic::error(
                    LabelErrorCode::InvalidImportedCitationForm,
                    &span.location,
                    "imported citation must be parenthesized",
                ));
            }
            continue;
        }
        if looks_imported(&span.content) {
            result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::InvalidImportedCitationForm,
                &span.location,
                "imported citation must use square brackets",
            ));
            continue;
        }
        let Ok(label) = Label::parse(span.content.trim(), LabelShape::Planning) else {
            continue;
        };
        let registry = match owner {
            MarkdownOwner::Plan => &mut result.registries.plan,
            MarkdownOwner::Doc => &mut result.registries.doc,
        };
        match span.context {
            InlineCodeContext::Bare => {
                let mint = LabelMint {
                    owner: owner.owner(),
                    label,
                    location: span.location.clone(),
                    home: span.home,
                };
                registry.insert_or_diagnose(mint, owner.name(), &mut result.diagnostics);
            }
            InlineCodeContext::Parenthesized => match owner {
                MarkdownOwner::Plan => {
                    result.push_same_owner_citation(LabelOwner::Plan, label, span.location);
                }
                MarkdownOwner::Doc => {
                    result.push_same_owner_citation(LabelOwner::Doc, label, span.location);
                }
            },
            InlineCodeContext::Asymmetric => result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::AsymmetricCitation,
                &span.location,
                "label citation has an unmatched parenthesis",
            )),
        }
    }
}
fn import(
    token: &str,
    location: &SourceLocation,
    source_owner: LabelOwner,
    result: &mut RepositoryLabels,
) {
    match ImportedLabel::parse(token) {
        Ok(value) => result.push_imported_citation(source_owner, value, location.clone()),
        Err(error) => result.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::UnknownOwner,
            location,
            error.to_string(),
        )),
    }
}
/// Harvest attestation citations from the realization body only.
///
/// Fenced code is ignored, and tokens inside the generated §17
/// upward-citation index are collected separately: the pinned
/// anchor-set hash and the imported-citation set derive from body
/// occurrences, so a token present only in the index cannot keep
/// itself in the release anchor set. The committed index is instead
/// welded to the body by set equality.
fn harvest_attestation_citations(path: &Path, source: &str, result: &mut RepositoryLabels) {
    let mut fence: Option<(char, usize)> = None;
    let mut in_index = false;
    let mut index_location = SourceLocation::new(path, 1, 1);
    let mut index_names: BTreeSet<String> = BTreeSet::new();
    for (number, line) in source.lines().enumerate() {
        if let Some((marker, length)) = fence {
            if fence_close(line, marker, length) {
                fence = None;
            }
            continue;
        }
        if let Some(open) = fence_open(line) {
            fence = Some(open);
            continue;
        }
        if line.starts_with("## ") {
            // Matched on the locator mint form so a heading merely
            // citing (`sec:anchors`) cannot open the index region.
            in_index = line.contains(" · `sec:realization:anchors`");
            if in_index {
                index_location = SourceLocation::new(path, number + 1, 1);
            }
        }
        let nonparticipating = nonparticipating_ranges(line);
        let mut offset = 0;
        while let Some(found) = line[offset..].find("[A-") {
            let start = offset + found;
            let body = start + "[A-".len();
            let Some(close) = line[body..].find(']') else {
                break;
            };
            let value = &line[body..body + close];
            let example = nonparticipating
                .iter()
                .any(|(from, to)| start >= *from && start < *to);
            if !example && let Ok(label) = Label::parse(value, LabelShape::Attestation) {
                if in_index {
                    index_names.insert(value.to_owned());
                } else {
                    result.attestation_anchor_names.push(value.to_owned());
                    result.push_imported_citation(
                        LabelOwner::Realization,
                        ImportedLabel {
                            owner: LabelOwner::Attestation,
                            label,
                        },
                        SourceLocation::new(path, number + 1, line[..start].chars().count() + 1),
                    );
                }
            }
            offset = body + close + 1;
        }
    }
    result.attestation_index_names = index_names;
    result.attestation_index_location = Some(index_location);
}

/// Byte ranges of inline code spans with a delimiter run of two or
/// more backticks on one line: nonparticipating example spans under
/// ADR-013, excluded from the raw anchor scan.
fn nonparticipating_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut ranges = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor] != b'`' {
            cursor += 1;
            continue;
        }
        let start = cursor;
        let length = backtick_run(bytes, cursor);
        cursor += length;
        let mut end = cursor;
        while end < bytes.len() && !(bytes[end] == b'`' && backtick_run(bytes, end) == length) {
            end += 1;
        }
        if end == bytes.len() {
            break;
        }
        if length >= 2 {
            ranges.push((start, end + length));
        }
        cursor = end + length;
    }
    ranges
}

fn backtick_run(bytes: &[u8], start: usize) -> usize {
    bytes[start..]
        .iter()
        .take_while(|value| **value == b'`')
        .count()
}

fn validate(result: &mut RepositoryLabels) {
    add_architecture_citations(result);
    let citations = std::mem::take(&mut result.pending_citations);
    let (graph, graph_node_by_id) =
        build_label_graph(&result.registries, citations, &mut result.diagnostics);
    result.graph = graph;
    result.graph_node_by_id = graph_node_by_id;
    validate_attestation_index(result);
    validate_attestation_anchor_pin(result);
}

/// Weld the committed §17 upward-citation index to the body anchor
/// set. Full-check only: the scoped derivations neither read nor
/// write the index, and a stale index must not block regenerating
/// registers or the model-label publication whose content does not
/// depend on it.
fn validate_attestation_index(result: &mut RepositoryLabels) {
    let Some(location) = result.attestation_index_location.clone() else {
        return;
    };
    let body_names: BTreeSet<String> = result.attestation_anchor_names.iter().cloned().collect();
    if result.attestation_index_names != body_names {
        result.diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::AttestationIndexStale,
            &location,
            "the committed upward-citation index does not present exactly the body's attestation anchor set",
        ));
    }
}
fn validate_attestation_anchor_pin(result: &mut RepositoryLabels) {
    let location = SourceLocation::new("packages/architecture/src/spec.rs", 1, 1);
    let architecture = &architecture::ARCHITECTURE;
    if let Some(pinned) = architecture.document.specification.anchor_set_hash {
        let actual = architecture::anchor_set_hash(
            result.attestation_anchor_names.iter().map(String::as_str),
        );
        if actual != pinned {
            result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::AttestationAnchorSetMismatch,
                &location,
                "the pinned attestation anchor-set hash does not match the realization's citations",
            ));
        }
    }
}

fn add_architecture_citations(result: &mut RepositoryLabels) {
    for witness in architecture::ARCHITECTURE.witnesses {
        match Label::parse(witness.semantic_tag, LabelShape::Realization) {
            Ok(label) => result.pending_citations.push(LabelCitation {
                source_owner: LabelOwner::Model,
                target: ImportedLabel {
                    owner: LabelOwner::Realization,
                    label,
                },
                origin: CitationOrigin::ArchitectureWitness(witness.id),
                class: CitationClass::SyntheticArchitecture,
            }),
            Err(error) => result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::InvalidLabel,
                &architecture_location(),
                error.to_string(),
            )),
        }
    }

    for clause in architecture::ARCHITECTURE.clauses {
        match Label::parse(clause.as_str(), LabelShape::Realization) {
            Ok(label) => result.pending_citations.push(LabelCitation {
                source_owner: LabelOwner::Model,
                target: ImportedLabel {
                    owner: LabelOwner::Realization,
                    label,
                },
                origin: CitationOrigin::ArchitectureClause(*clause),
                class: CitationClass::SyntheticArchitecture,
            }),
            Err(error) => result.diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::InvalidLabel,
                &architecture_location(),
                error.to_string(),
            )),
        }
    }
}

fn architecture_location() -> SourceLocation {
    SourceLocation::new("packages/architecture/src/spec.rs", 1, 1)
}

/// Build the direct Petgraph label graph from typed registries and citations.
pub fn build_label_graph(
    registries: &RegistrySet,
    citations: impl IntoIterator<Item = LabelCitation>,
    diagnostics: &mut Vec<LabelDiagnostic>,
) -> (
    DiGraph<LabelGraphNode, LabelGraphEdge, u32>,
    BTreeMap<LabelGraphNodeId, NodeIndex<u32>>,
) {
    let mut nodes = registry_mints(registries)
        .into_iter()
        .map(LabelGraphNode::Mint)
        .collect::<Vec<_>>();
    let mut citations = citations.into_iter().collect::<Vec<_>>();

    citations.sort_by(|left, right| {
        (
            &left.origin,
            &left.target.owner,
            &left.target.label,
            left.class,
        )
            .cmp(&(
                &right.origin,
                &right.target.owner,
                &right.target.label,
                right.class,
            ))
    });

    for citation in citations {
        if citation.class == CitationClass::AuthoredImported
            && citation.source_owner == citation.target.owner
        {
            diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::InvalidImportedCitationForm,
                &citation_location(&citation),
                "same-owner citation must use the local parenthesized form",
            ));
            continue;
        }

        nodes.push(LabelGraphNode::Citation(citation));
    }

    nodes.sort_by_key(LabelGraphNode::id);

    let mut graph =
        DiGraph::<LabelGraphNode, LabelGraphEdge, u32>::with_capacity(nodes.len(), nodes.len());
    let mut node_by_id = BTreeMap::new();

    for node_weight in nodes {
        let id = node_weight.id();

        if node_by_id.contains_key(&id) {
            report_duplicate_graph_node(&node_weight, diagnostics);
            continue;
        }

        let node = graph.add_node(node_weight);
        node_by_id.insert(id, node);
    }

    let mint_nodes = node_by_id
        .iter()
        .filter_map(|(id, node)| match id {
            LabelGraphNodeId::Mint { owner, label } => {
                Some(((owner.clone(), label.clone()), *node))
            }
            LabelGraphNodeId::Citation { .. } => None,
        })
        .collect::<BTreeMap<_, _>>();
    let mut pending_edges = Vec::new();

    for node in graph.node_indices() {
        let LabelGraphNode::Citation(citation) = &graph[node] else {
            continue;
        };
        let target_key = (citation.target.owner.clone(), citation.target.label.clone());

        match mint_nodes.get(&target_key).copied() {
            Some(target) => {
                pending_edges.push((graph[node].id(), node, graph[target].id(), target));
            }
            None => report_unresolved_citation(citation, diagnostics),
        }
    }

    pending_edges.sort_by(|left, right| (&left.0, &left.2).cmp(&(&right.0, &right.2)));

    for (_, source, _, target) in pending_edges {
        graph.add_edge(source, target, LabelGraphEdge::ResolvesTo);
    }

    if is_cyclic_directed(&graph) {
        diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::InvalidLabel,
            &SourceLocation::new("<label-graph>", 1, 1),
            "documentation label graph contains an impossible cycle",
        ));
    }

    validate_citation_outdegrees(&graph, diagnostics);

    (graph, node_by_id)
}

fn registry_mints(registries: &RegistrySet) -> Vec<LabelMint> {
    registries
        .attestation
        .iter()
        .map(|(_, mint)| mint.clone())
        .chain(registries.realization.iter().map(|(_, mint)| mint.clone()))
        .chain(
            registries
                .adrs
                .values()
                .flat_map(|registry| registry.iter().map(|(_, mint)| mint.clone())),
        )
        .chain(registries.model.iter().map(|(_, mint)| mint.clone()))
        .chain(registries.plan.iter().map(|(_, mint)| mint.clone()))
        .chain(registries.doc.iter().map(|(_, mint)| mint.clone()))
        .chain(
            registries
                .crates
                .values()
                .flat_map(|registry| registry.iter().map(|(_, mint)| mint.clone())),
        )
        .collect()
}

fn citation_location(citation: &LabelCitation) -> SourceLocation {
    match &citation.origin {
        CitationOrigin::Source { location, .. } => location.clone(),
        CitationOrigin::ArchitectureWitness(_) | CitationOrigin::ArchitectureClause(_) => {
            architecture_location()
        }
    }
}

fn report_duplicate_graph_node(node: &LabelGraphNode, diagnostics: &mut Vec<LabelDiagnostic>) {
    let location = match node {
        LabelGraphNode::Mint(mint) => mint.location.clone(),
        LabelGraphNode::Citation(citation) => citation_location(citation),
    };

    diagnostics.push(LabelDiagnostic::error(
        LabelErrorCode::InvalidLabel,
        &location,
        "duplicate documentation label graph node",
    ));
}

fn report_unresolved_citation(citation: &LabelCitation, diagnostics: &mut Vec<LabelDiagnostic>) {
    let location = citation_location(citation);
    let (code, message) = match citation.class {
        CitationClass::AuthoredSameOwner => (
            LabelErrorCode::MissingMint,
            format!("unresolved same-owner citation {}", citation.target.label),
        ),
        CitationClass::AuthoredImported => (
            LabelErrorCode::UnknownImportedLabel,
            format!(
                "unresolved imported label {}{}",
                citation.target.owner.prefix(),
                citation.target.label,
            ),
        ),
        CitationClass::SyntheticArchitecture => (
            LabelErrorCode::ArchitectureLabelMissing,
            format!(
                "architecture manifest label {} is missing from the realization",
                citation.target.label,
            ),
        ),
    };

    diagnostics.push(LabelDiagnostic::error(code, &location, message));
}

fn validate_citation_outdegrees(
    graph: &DiGraph<LabelGraphNode, LabelGraphEdge, u32>,
    diagnostics: &mut Vec<LabelDiagnostic>,
) {
    for node in graph.node_indices() {
        let LabelGraphNode::Citation(citation) = &graph[node] else {
            continue;
        };
        let count = graph.edges_directed(node, Direction::Outgoing).count();

        if count > 1 {
            diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::MissingMint,
                &citation_location(citation),
                format!("citation resolves to {count} label mints; expected exactly one"),
            ));
        }
    }
}

/// Project a direct Petgraph label graph into stable typed values.
#[must_use]
pub fn project_label_graph(
    graph: &DiGraph<LabelGraphNode, LabelGraphEdge, u32>,
) -> LabelGraphProjection {
    let mut nodes = graph
        .node_weights()
        .map(LabelGraphNode::id)
        .collect::<Vec<_>>();
    nodes.sort();

    let mut edges = graph
        .edge_references()
        .map(|edge| LabelGraphEdgeProjection {
            source: graph[edge.source()].id(),
            target: graph[edge.target()].id(),
            edge: *edge.weight(),
        })
        .collect::<Vec<_>>();
    edges.sort();

    LabelGraphProjection { nodes, edges }
}

fn square(value: &str) -> Option<&str> {
    value.strip_prefix('[')?.strip_suffix(']')
}
fn looks_imported(value: &str) -> bool {
    ImportedLabel::parse(value).is_ok()
}
#[derive(Clone, Debug)]
pub struct GeneratedRegister {
    pub path: PathBuf,
    pub bytes: usize,
}
#[derive(Debug, Error)]
pub enum GenerateError {
    #[error("label source validation failed")]
    Validation(Vec<LabelDiagnostic>),
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON rendering failed: {0}")]
    Json(#[from] serde_json::Error),
}
impl GenerateError {
    pub fn diagnostics(&self) -> &[LabelDiagnostic] {
        match self {
            Self::Validation(diagnostics) => diagnostics,
            _ => &[],
        }
    }
}
/// Generate the specification and realization registers from their owning
/// upstream sources only, writing to the argument-supplied output
/// paths (ADR-014: assets go only where arguments route them).
///
/// An unrelated planning or ADR defect must not block regenerating an
/// upstream register (ADR-013 scoped-derivation rule);
/// `check_repository` remains the full repository-wide gate.
pub fn generate_registers(
    paths: &RepositoryCensus,
    specification_output: &Path,
    realization_output: &Path,
) -> Result<Vec<GeneratedRegister>, GenerateError> {
    let labels = derive_model_sources(paths);
    if labels.has_errors() {
        return Err(GenerateError::Validation(labels.diagnostics));
    }
    let outputs = [
        (
            specification_output.to_path_buf(),
            render::specification_register(&labels.registries.attestation),
        ),
        (
            realization_output.to_path_buf(),
            render::realization_register(&labels.registries.realization),
        ),
    ];
    let mut written = Vec::new();
    for (path, contents) in outputs {
        write(&path, contents.as_bytes())?;
        written.push(GeneratedRegister {
            path,
            bytes: contents.len(),
        });
    }
    Ok(written)
}
pub fn model_labels_json(paths: &RepositoryCensus) -> Result<String, GenerateError> {
    let labels = derive_model_sources(paths);
    if labels.has_errors() {
        return Err(GenerateError::Validation(labels.diagnostics));
    }
    Ok(render::model_labels_json(&labels.registries.model)?)
}

fn derive_model_sources(paths: &RepositoryCensus) -> RepositoryLabels {
    let mut result = RepositoryLabels::default();
    // Scoped census verification (ADR-014): a stale plan or ADR census
    // must not block an upstream derivation, but a stale scoped census
    // would silently change the derived registers.
    result
        .diagnostics
        .extend(paths.verify(crate::census::CensusGroup::SCOPED));
    let (attestation, diagnostics) = harvest_attestation(paths);
    result.registries.attestation = attestation;
    result.diagnostics.extend(diagnostics);
    harvest_realization(paths, &mut result);
    add_model(harvest_model(paths), &mut result);
    result
        .pending_citations
        .retain(scoped_citation_participates);
    let citations = std::mem::take(&mut result.pending_citations);
    let (graph, graph_node_by_id) =
        build_label_graph(&result.registries, citations, &mut result.diagnostics);
    result.graph = graph;
    result.graph_node_by_id = graph_node_by_id;
    sort_diagnostics(&mut result.diagnostics);
    result
}

const fn scoped_citation_participates(citation: &LabelCitation) -> bool {
    matches!(
        citation.source_owner,
        LabelOwner::Attestation | LabelOwner::Realization | LabelOwner::Model
    ) && matches!(
        citation.target.owner,
        LabelOwner::Attestation | LabelOwner::Realization | LabelOwner::Model
    )
}
fn write(path: &Path, bytes: &[u8]) -> Result<(), std::io::Error> {
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(directory)?;
    let mut staged = tempfile::Builder::new()
        .prefix(".labels-staged-")
        .tempfile_in(directory)?;
    staged.write_all(bytes)?;
    staged.as_file().sync_all()?;
    staged
        .persist(path)
        .map(|_| ())
        .map_err(|error| error.error)
}
