# Proposal: a relay-admissible maturity announcement witness by constant elision (shape A)

Status: the design proposal ratified by ADR-025, 2026-09-22. It asks for one ruling and opens no route by itself. Every quantitative fact below is either measured in the tree at `0.6.232-dev` or recomputed in the relay-witness study (STUDY-relay-split.md, sections 1 to 7); estimates are marked as such.

## 0. In one breath

Carry only the 53 variable bytes of the predecessor metadata in the witness, let the executing leaf re-materialize the 25-byte constant header from a linker-resolved symbol and the 8 reserved zero bytes from a literal before running its unchanged checks, and make the target's argument-width policy an input to emission so that a type with no legal transport fails to compile instead of emitting a relay-refused program.

## 1. The problem, exactly

The announcement's initial witness has nine items; the executing script and the control block are removed before the relay policy check and the remaining seven are each limited to 80 bytes by `MAX_STANDARD_TAPSCRIPT_STACK_ITEM_SIZE` (Elements `src/policy/policy.h:56`, applied in `policy.cpp:310–330`). Exactly one item violates it: index 4, the canonical predecessor metadata, is 86 bytes. Consensus allows 520-byte elements during execution (`src/script/script.h:32`), so the 86-byte and 118-byte intermediates the leaf computes are legal; only the initial argument is not. Tripod pins the same 80 in `packages/target-elements/src/transaction_form.rs:252–283` and reviews the current form as consensus-admitted and relay-refused; the ABI reads that review and publishes the residual `RelayAdmissibleWitnessSplitUnopened` (`packages/transaction/src/state_abi.rs:517–561`).

The 86 bytes are 21 domain bytes (`tripod/state-metadata`), 4 schema bytes (big-endian 1), five 8-byte semantic fields, the 9-byte maturity encoding, the 4-byte representation nonce and 8 reserved zero bytes (`packages/realization/src/state_codec.rs:132–155`). The constant regions total 33 bytes at offsets 0..25 and 78..86; the variable region is the contiguous span 25..78, 53 bytes.

The executing leaf authenticates all 86 bytes: it streams them into the `TapLeaf/elements` hash, places the resulting digest against the retained static root under `TapBranch/elements`, tweaks the internal key under `TapTweak/elements` and checks the introspected output key (`packages/tapscript/src/state_announcement.rs:391–446`). Every later fragment slices the 86 bytes at fixed offsets; the copy-through fragment already synthesizes the successor's reserved zeros in-script (`state_announcement.rs:516–532`).

## 2. The principle

The layering is the ordinary compiler one. The frontend (`realization`) defines the type and states facts about it; it never learns a target limit. The backend (`tapscript`) lowers the type to the target and owns every transport decision; when no legal transport exists it refuses with a typed error at emission. The linker (`linker`) resolves constants into the program and re-measures what the lowering asserted. The ABI (`transaction`) is a derivation from the lowered record, never a second table. The loader (`transaction::state_signing`) populates the frame from the same definition the emitter used, and every consumer of the bytes reconstructs by that definition too.

Constants therefore never travel in the argument frame. The domain, the schema and the reserved zeros are constants of the type under this constructor generation; the linker already relocates six constants into the leaf (asset, amount, internal key, both lead bounds, the operator key: `packages/linker/src/tests/state_symbol_tests.rs:190–233`), and the header joins them.

## 3. The design

### 3.1 Frontend: classify the rows, change nothing else

The only frontend change is a classification on the layout rows: each row of the canonical encoding is marked constant, with its value, or variable. Encoding and decoding are unchanged and remain strict; the strict decoder keeps refusing a wrong domain, a wrong schema and non-zero reserved bytes (`state_codec.rs:174–221`), so the classification states what the codec already enforces.

```rust
pub enum RegionClass { Constant(&'static [u8]), Variable }
pub struct LayoutRow { pub name: &'static str, pub range: Range<usize>, pub class: RegionClass }
pub const LAYOUT: [LayoutRow; 9] = [
    row("domain",   0..21,  Constant(b"tripod/state-metadata")),
    row("schema",   21..25, Constant(&[0, 0, 0, 1])),
    row("omega",    25..33, Variable), row("y_l", 33..41, Variable), row("y_t", 41..49, Variable),
    row("q",        49..57, Variable), row("cycle", 57..65, Variable),
    row("maturity", 65..74, Variable), row("nonce", 74..78, Variable),
    row("reserved", 78..86, Constant(&[0; 8])),
];
```

### 3.2 Backend: target policy in, legalization out

The target description becomes an input to emission. The composed record is built with a witness schedule and the policy; the lowering derives, from the frontend's classification, the one legal transport for the metadata role and refuses when none exists.

```rust
pub struct TargetPolicy { pub max_initial_item: usize /* 80 */, pub max_element: usize /* 520 */, pub max_stack: usize /* 1000 */, pub max_weight: u64 /* 400_000 */ }

pub enum StateWitnessSchedule { WholeMetadata /* historical, replay-only */, VariableMetadata }

pub enum Legalization {
    Whole { width: usize },
    ExpandFromVariable { carried: Range<usize>, prefix: Symbol, suffix_literal: &'static [u8], whole: usize },
}

fn expand_from_variable(layout: &[LayoutRow], policy: &TargetPolicy) -> Result<Legalization, ScheduleRefusal> {
    let variable: Vec<&LayoutRow> = layout.iter().filter(|r| matches!(r.class, RegionClass::Variable)).collect();
    if variable.windows(2).any(|w| w[0].range.end != w[1].range.start) { return Err(ScheduleRefusal::VariableRowsNotContiguous); }
    let span = variable[0].range.start .. variable[variable.len() - 1].range.end;          // 25..78
    if span.len() > policy.max_initial_item { return Err(ScheduleRefusal::NoLegalTransport { width: span.len(), limit: policy.max_initial_item }); }
    let prefix = concat_constants(layout.iter().take_while(|r| r.range.end <= span.start));   // 25 bytes -> Symbol::MetadataHeader
    let suffix = concat_constants(layout.iter().skip_while(|r| r.range.start < span.end));    // 8 zero bytes -> literal
    Ok(Legalization::ExpandFromVariable { carried: span, prefix: Symbol::from_bytes(prefix), suffix_literal: suffix, whole: 86 })
}

pub fn compose_state_program(deployment: &Deployment, schedule: StateWitnessSchedule, policy: &TargetPolicy) -> Result<ComposedStateProgram, ScheduleRefusal> {
    let legalization = match schedule {
        StateWitnessSchedule::WholeMetadata    => Legalization::Whole { width: 86 },          // never selectable for a new deployment
        StateWitnessSchedule::VariableMetadata => expand_from_variable(&LAYOUT, policy)?,
    };
    …
}
```

The emitted change is a prologue at the entry of the metadata-authentication fragment, generated from the legalization; everything after it is byte-identical to today, including the existing packing of root and metadata beneath the prefix (`state_announcement.rs:421–446`).

```text
stack on entry, deepest first:  root, variable(53), prefix
SWAP                            root, prefix, variable
PUSH MetadataHeader (symbol)    root, prefix, variable, header(25)
SWAP; CAT                       root, prefix, header||variable (78)
PUSH 0x00*8 (literal); CAT      root, prefix, canonical(86)
SWAP                            root, canonical, prefix        == today's entry state
```

Seven instructions, 40 script bytes. The role table keeps seven entries; only the metadata role's width and source change (86 and canonical encoding under the historical schedule; 53 and canonical variable region under the new one).

### 3.3 Linker: one symbol, one measurement

`MetadataHeader` becomes the seventh pushed symbol, resolved from the frontend's constant rows and checked for its 25-byte width at resolution; a mismatch is a link error. The relocation census moves from six pushed symbols at fifteen sites to seven at sixteen; the twelve sites after the prologue shift by seven instructions (`packages/linker/src/tests/state_relocate_tests.rs:272–295`). Carrier attribution records the header commitment as carried by the linked program and the reserved zeros as a leaf literal; the witness carries the variable region only.

The resource measurement (`packages/linker/src/state_resource.rs:101–183`) gains one comparison it deliberately lacks today: each initial argument width against the target policy's 80, kept separate from the 520-byte execution bound so the leaf's legitimate 86- and 118-byte intermediates are not banned. A non-historical schedule with any initial item over the limit is a link refusal; the lowering has already refused it, and the linker re-measures so that a table edit cannot lie.

### 3.4 ABI, loader, consumers

The ABI is derived from the record as it is today (`state_abi.rs:863–933`); the transaction form review becomes per schedule, the historical refusal stays on record, and the split residual is discharged only for a schedule the linker measured admissible. The loader slices the one encoder result by the legalization's carried range (`packages/transaction/src/state_signing.rs:935–973`). The projector landed by `T11-093` and extended by `T11-094` reconstructs by the same legalization: it takes the schedule from the retained recipe, treats the observed width as a check, rebuilds the canonical 86 bytes and decodes them with the one strict decoder. Two safety-matrix rows, header tampering and reserved-byte tampering, become unrepresentable at the witness carrier; they keep their evidence at the codec's refusal tests and at the leaf-script binding, and the matrix annotates the carrier move rather than deleting the rows.

### 3.5 Evidence

The archived refused run stays admitted exactly as it is, replayed under the historical schedule that the planner takes as an input. The new schedule produces a new leaf, hence a new static root, output key and control; an old-tree predecessor cannot spend into a new-tree successor (`packages/linker/src/state_bundle.rs:373–409`), so accepted evidence needs a fresh funded predecessor and a fresh run through the existing executor and schema 8, which already requires an accepted identity and a mined readback on acceptance (`packages/target-elements-conformance/src/protocol.rs:3208–3225`). That run lands in a second immutable four-file corpus with its own pinned importer; the acceptance obligation gains an established variant carrying the schedule, identity and readback; the executor's fee-free environment (`-minrelaytxfee=0`) is disclosed beside it, so the result proves acceptance in that environment, not positive-fee relayability.

## 4. Alternatives considered

| Shape | Transport | Why not |
|---|---|---|
| B: two pieces 78 + 8, four-instruction join | smallest patch; ten items | passes constants as arguments; adds a witness role; the minimal join enforces no partition |
| C: shorten the codec to 78 | no prologue | changes the committed language, decoder, commitments and goldens for a six-byte excess |
| D: carry 78, synthesize the zeros | four instructions, seven roles | half-principled: header passed, padding synthesized |
| G: carry 61, synthesize the header | five instructions | reserved bytes travel although copy-through already freezes them at zero on the output side |
| E, F: other cuts, more pieces | as B | as B, with more roles |
| H: B plus SIZE guards | enforces the partition | more transport checks for no correctness gain |
| I: another leaf version | escapes the policy check | an unreviewed execution contract |
| J: relax the node's policy | configuration | one node's loader; the network keeps 80 |
| K: block-layer submission subject | bypass admission | never relay-admissible; needs a protocol revision and its own study |
| L: the annex | | refused outright by policy |
| M: bake the argument into the code | | a state-specific leaf breaks the shared static subtree and the relocation model |

A is the only shape under which constants live in code on both sides of the transition and the frontend's classification, not a hand-chosen offset, defines the transport.

## 5. Sizes and counts

| Quantity | Today | Shape A |
|---|---|---|
| Initial item widths | [1, 4, 8, 32, 86, 1, 64] | [1, 4, 8, 32, 53, 1, 64] |
| Executing leaf bytes (archive) | 1,286 | about 1,326 (estimate) |
| Encoded instructions (archive census) | 475 | 482 (estimate) |
| Serialized transaction bytes / weight | 1,694 / 2,084 | about 1,701 / 2,091 (estimate) |
| Largest initial item, headroom under 80 | 86, none | 53, 27 bytes |
| Pushed relocation symbols / sites | 6 / 15 | 7 / 16 |
| Public witness roles | 7 | 7 |
| Committed canonical bytes | 86 | 86, unchanged |
| Signature and curve checks, validation budget | 3, 150 | unchanged |

## 6. Blast radius

The study counted, for shape B, 15 production files, 17 test or fixture files, five documentation or driver files and one census file, 38 in all. Shape A leaves the two graph and carrier production files and their two count tests as unchanged consumers (34) and adds the codec's layout table with its tests and the linker's symbol census with its tests (four): about 38 existing files, plus four new immutable evidence files. The constructor arithmetic, the semantic transition, the codec and the six field-range laws stay textually unchanged; every derived golden is recomputed with real arithmetic and independent framing, none by guess (`packages/tapscript/src/tests/state_constructor_tests.rs:22–28,1230–1263`).

## 7. Bite plan

Each bite is one lane, one named commit, gated. Estimates are changed or added lines, not measurements.

1. Route record and allocation, node-free: the ruling, the route rows from the first free number at landing (proposed `T11-100`), `T11-098`'s dependency, Wave 12's resource ownership. Prose 100 to 180 lines.
2. Frontend classification, node-free: the layout rows with their classes in `realization::state_codec`, derived helpers for the variable region and the canonical rebuild, tests that the classification and the encoder agree byte for byte. Production 60 to 120, tests 80 to 150.
3. Historical schedule selection, node-free: the schedule as a retained field of the composed record, taken by the planner, the closure and the corpus loader; the archived run replays byte-identically under it; cross-schedule replay refuses. Production 180 to 300, tests 150 to 250.
4. Legalization, prologue, symbol and loader, node-free, atomic: the target policy object, the lowering with its two refusals, the generated prologue, the `MetadataHeader` symbol with its width check, the loader's slice, the derived ABI and per-schedule form review. Gate: the seven-item map with 53 at index 4; every initial item at most 80; old and new stacks identical after the prologue; unchanged semantic outcomes; wrong, short and long variable items refused; strict round trips and complete-tuple determinism. Production 250 to 400, tests 350 to 600.
5. Policy and resource accounting, node-free: the initial-argument comparison beside the execution bound; recomputed script, instruction, initial and peak stack figures; the historical refusal retained. Production 140 to 240, tests 180 to 300.
6. Constructor, relocation and golden sweep, node-free: schedule-aware fixtures, seven goldens and both sides' nonces recomputed with real arithmetic, sixteen relocation sites, old and new trees failing continuity. Production 30 to 70, tests 300 to 500.
7. Accepted capture and admission contract, node-free: the established acceptance variant, the second pinned corpus and importer, the old loader untouched. Production 400 to 650, tests 400 to 650.
8. Continuity integration, node-free: the projector reconstructs by legalization from the retained schedule; the two carrier-moved matrix rows annotated; the continuity report reads the schedule. Production 120 to 220, tests 200 to 350.
9. Fresh node run and immutable admission, node run: a newly funded predecessor under the new leaf, the three exchanges through the existing driver and executor, acceptance with exact mined bytes, txid and wtxid, admitted into the second corpus; a refusal is retained and diagnosed, never renamed. Fixture data 150 to 220 lines plus the capture.
10. Closure and handoff, node-free: measured values, evidence identities, row dispositions, remaining obligations, `T11-098`'s status. Prose 80 to 140.

Bites 2 to 4 are the difference from the study's plan, which had one bite for a two-piece join; everything from 5 onward is the study's plan with the shape substituted.

## 8. What this does not establish

No relayability under a positive fee floor, no root freshness, no Wave 12 resource closure and no artifact promotion; acceptance in the executor's environment is what the fresh run can show. Shape A's equivalence to today's authentication holds over canonical states, which the codec enforces at construction; a non-canonical predecessor that the current leaf would authenticate is refused under A, which is the stricter and intended behaviour. The header and reserved rows lose their witness carrier in the safety matrix and keep their codec and leaf-binding evidence. Goldens, selected nonces, output keys, controls and signature messages all change with the leaf and are recomputed, not carried over.

## 9. The ruling requested

Adopt as the next Wave-9 ruling, on the merits:

1. The route to accepted announcement bytes is shape A: the witness carries the variable region of the predecessor metadata only; the executing leaf re-materializes the constant regions before its unchanged checks.
2. The frontend's only change is the classification of layout rows as constant or variable; it states no limit and learns none.
3. The target's argument-width policy is an input to emission: the backend derives the transport from the classification, and a type with no legal transport is a compile error, never an emission reviewed as relay-refused.
4. Constants are linked, never passed: the header is a relocated linker symbol checked for width at resolution; the reserved zeros are a leaf literal, as on the successor side today.
5. The historical whole-item schedule remains constructible for exact replay of the archived run and is never selectable for a new deployment.
6. The route is its own track with rows allocated from the first free number at landing; `T11-098` depends on its accepted evidence; Wave 9's node-free work continues unchanged.
7. Accepted evidence requires a newly funded predecessor and a fresh run into a second immutable corpus under the existing executor and schema; the fee-free environment is disclosed; no artifact is promoted by acceptance alone.

## 10. Points the review may want to settle

Whether the header is one 25-byte symbol (proposed) or two symbols, domain and schema. Whether the reserved zeros should also be a symbol rather than a literal (proposed: literal, matching copy-through). Whether bite 9's run waits for the Wave-9 closure or proceeds as soon as bites 1 to 8 are green (proposed: as soon as green, on a node free of lanes and gates).
