# ADR-012: Documentation Labels and Cross-Reference Layers

**Status:** Implemented; the unlinted-local-label and lint-boundary decisions
are superseded by [ADR-013](013-label-calculus.md)
**Scope:** Markdown under `adr/` and `plans/`, plus upstream-label registers and documentation checks
**Implemented by:** `packages/labels` and `scripts/check-plans.sh`

---

## Context · `sec:labels:context`

The repository has several documentation layers with different authority:

1. Layer 0;
2. the realization document;
3. implemented ADRs;
4. non-normative implementation plans.

All layers need stable human cross-references. They must not be treated as one
namespace or one authority class.

The realization document already distinguishes:

- a local label mint;
- an internal citation;
- an upstream citation in square brackets.

Planning documents adopt the same visual rule one layer lower.

The distinction is semantic:

- a plan-local label is navigation inside non-normative planning;
- a square-bracket citation imports a fact from an upstream authority.

Planning Markdown and planning labels remain outside protocol, compiler,
bundle, ABI, evidence, and release identities.

---

## Decision · `rule:labels:decision`

Documentation uses labeled, layered cross-references.

A label has an owner and a local name. Its citation form states whether the
owner is the current planning layer or an upstream authority.

The three forms are:

```text
local mint:       `pkg:realization:contract`
local citation:   (`pkg:realization:contract`)
upstream citation (`[RZ-sec:realization:representation]`)
```

Square brackets mean:

> this citation imports an upstream authority.

Backticks without square brackets mean:

> this is a plan-local navigation label.

---

## Label shape · `rule:labels:shape`

The normal label shape is:

```text
<type>:<area>:<name>
```

Segments use lowercase ASCII letters, digits, and hyphens.

Examples:

```text
sec:plans:authority
rule:plans:generated-direction
dec:toolchain:typed-rust
pkg:realization:contract
phase:roadmap:realization
task:realization:create-crate
gate:realization:exit
q:arithmetic:wide-floor
ref:elements:tapscript
tbl:packages:index
rem:release:non-normative
```

Upstream owners may define additional shapes. In particular, Layer 0 and
Realization retain their existing two-segment `sec:*` and `app:*` labels.

Planning labels should use the three-segment form.

---

## Local planning labels · `rule:labels:local`

A planning document mints a local label as a bare inline code span at the
label’s one conceptual home:

```markdown
## Typed inputs · `sec:realization:inputs`
```

An internal planning citation uses the same label in parentheses:

```markdown
The compiler consumes the typed declaration
(`pkg:realization:contract`).
```

Plan-local labels are:

- non-normative;
- navigation-only;
- unlinted;
- excluded from every semantic identity and hash;
- freely renameable without protocol or release versioning;
- never compiler, linker, transaction, evidence, or release input.

The documentation checker does not enforce plan-local label existence,
uniqueness, completeness, or citation resolution.

Normal Markdown file links remain linted independently.

---

## Imported labels · `rule:labels:external-citation`

An imported label is wrapped in square brackets inside its inline code span:

```markdown
The representation policy is inherited from
(`[RZ-sec:realization:representation]`).
```

The complete citation form is:

```text
(`[<owner>-<upstream-label>]`)
```

Imported labels are linted for:

- recognized owner;
- valid owner-specific shape;
- existence in the upstream owner’s register;
- correct square-bracket citation form.

The upstream artifact remains authoritative. A generated register is an index,
not a second source.

The presence of a valid citation proves only that the upstream label exists.
It does not prove that surrounding planning prose interprets it correctly.

---

## Owner prefixes · `rule:labels:owners`

The initial imported-label owners are:

| Prefix | Owner |
|---|---|
| `A-` | Attestation Layer 0 |
| `RZ-` | Attestation Realization |
| `ADR010-` | ADR-010 command-line output policy |
| `ADR011-` | ADR-011 toolchain and dependency policy |
| `ADR012-` | This documentation-label policy |

Examples:

```text
(`[A-def:model:classes]`)
(`[RZ-def:versioning:denotation-law]`)
(`[RZ-obl:oracle:ledger]`)
(`[ADR010-rule:output:streams]`)
(`[ADR011-rule:toolchain:locked]`)
(`[ADR012-rule:labels:external-citation]`)
```

A future owner prefix requires an ADR-012 update or another ADR that extends
this registry.

---

## Normativity classes · `tbl:labels:authority`

| Label class | Authority | Linted | Identity-bearing |
|---|---|---:|---:|
| Plan-local label | Planning navigation only | No | No |
| ADR label | Implemented repository policy | Yes when imported | ADR-owned only |
| Realization label | Normative realization contract | Yes | Upstream-owned |
| Layer-0 label | Normative Layer-0 contract | Yes | Upstream-owned |
| Generated label register entry | Derivative index | Checked for freshness | No independent identity |

A plan citing an upstream label does not make the plan normative. The cited
upstream fact retains its own authority.

---

## Mint discipline · `rule:labels:mint`

Every independently citable planning unit should mint one local label.

This includes:

- sections;
- planning decisions;
- package contracts;
- roadmap phases;
- backlog tasks;
- exit gates;
- research questions;
- candidate constructions compared by research;
- reference claim tables;
- rules and remarks cited elsewhere.

Do not mint labels for transitional prose or one-use examples.

One conceptual fact should have one planning home. Other planning documents
cite that home rather than restating it.

---

## Upstream registers · `rule:labels:registers`

The planning tree carries generated upstream-label registers:

```text
plans/labels/specification.md
plans/labels/realization.md
```

`specification.md` is generated from Layer-0 LaTeX `\label{...}` declarations.

`realization.md` is generated from label mints in:

```text
docs/attestation/realization.md
```

An upstream register contains:

- imported citation token;
- owner-local label;
- source home or heading;
- generation notice.

Registers are:

- deterministic;
- sorted canonically;
- non-normative derivatives;
- checked without writing in CI;
- never semantic or compiler input.

ADRs are small enough that their labels may be harvested directly from the ADR
source instead of through a committed aggregate register.

---

## Realization mint rule · `rule:labels:realization-harvest`

The Realization harvester follows that document’s citation contract.

Outside fenced code blocks:

- bare valid label code spans are mints;
- round-parenthesized label code spans are internal citations;
- square-bracketed label code spans are external citations;
- ordinary code spans are ignored.

Examples:

```text
`sec:representation`          mint
(`sec:representation`)        internal citation
(`[A-def:model:classes]`)    external citation
```

The harvester requires:

- balanced inline-code delimiters;
- one mint for each v13-owned label;
- no duplicate mint;
- every internal v13 citation resolves;
- canonical owner-specific label shape.

The generated plan register exposes those labels under the `RZ-` consumer
prefix.

---

## Planning lint boundary · `rule:labels:lint-boundary`

The planning checker validates:

- directory README ownership;
- Markdown file indexes;
- relative links;
- generated upstream-register freshness;
- imported upstream-label citations;
- recognized owner prefixes;
- documentation size budgets;
- selected planning hygiene rules.

It does not validate:

- plan-local label existence;
- plan-local label uniqueness;
- plan-local citation resolution;
- semantic correctness of planning prose.

A square-bracket citation using a plan-local prefix or plan-local label is a
form error because square brackets claim upstream authority.

---

## Generated/check split · `rule:labels:generation`

Updating a register and checking a register are separate operations.

Update mode may write:

```text
plans/labels/specification.md
plans/labels/realization.md
```

Check mode:

- computes expected bytes in memory;
- compares committed bytes;
- reports staleness;
- never writes.

CI and tests use check mode only.

---

## Machine-use prohibition · `rule:labels:no-semantic-input`

No semantic package may consume:

- planning labels;
- planning Markdown;
- generated planning label registers;
- ADR Markdown;
- documentation-check output.

This prohibition applies to:

- architecture;
- realization;
- model behavior;
- compiler;
- target packages;
- backends;
- linker;
- transaction construction;
- vectors;
- release validation.

Typed source and explicit typed configuration implement policy. Documentation
explains and cross-references it.

---

## Consequences · `sec:labels:consequences`

The policy provides:

- one visual distinction between local navigation and imported authority;
- v13-style citations one layer lower;
- linted normative references without making plans normative;
- dense cross-references instead of repeated prose;
- deterministic upstream-label indexes;
- freedom to reorganize plan-local labels during implementation.

The policy deliberately permits undetected typos in plan-local label
references. That is the cost of keeping planning labels non-normative and
outside CI identity discipline.

Markdown links remain available when guaranteed navigation is required.

---

## Rejected alternatives · `sec:labels:alternatives`

### Lint every planning label

Rejected because it would turn temporary planning navigation into a
repository-stable identifier system and add maintenance without semantic
assurance.

### Put every planning label in a global registry

Rejected because plan-local labels are intentionally lightweight and
non-normative.

### Cite upstream labels without an owner prefix

Rejected because Layer 0, Realization, ADRs, and plans may use overlapping
label text.

### Use only Markdown links

Rejected because links identify files and headings but do not express the
authority class of the cited proposition.

### Parse planning labels as compiler configuration

Rejected because plans are not semantic input.

---

## Verification · `gate:labels:implementation`

ADR-012 is implemented when:

- every documentation directory has a `README.md`;
- Layer-0 and v13 register generators exist;
- check mode is non-writing;
- every imported plan/ADR citation resolves;
- unknown owner prefixes fail;
- plan-local square-bracket citations fail;
- plan-local labels are otherwise ignored by the linter;
- register output is deterministic;
- combined `plans/` and `adr/` Markdown stays within the accepted weight cap;
- `scripts/check-plans.sh` passes;
- the full repository CI gate remains green.
