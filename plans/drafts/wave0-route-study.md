# Research: routes for the Wave 0 ABI entries and residual Safety obligations

**Abstract (Status and scope)** · `abst:w0route:scope`

This design study proposes how the ABI-entries step and the residual-routes step of bite 0.5 can be built, sized and ordered, and briefly how the five native construction routes can be captured. It binds nothing and records no performed work: every entry, test, archive and record named below is a proposal for a later producing step, and the guide's gates remain the only acceptance surface (`task:guide15-exec:bite-0-5`), (`gate:guide15-exec:bite-0-5-gate`), (`def:guide15-exec:retired-residual`), (`rule:drafts:authority`). The four step rows (`step:wave0:bite-0-5-first-party-entries`), (`step:wave0:bite-0-5-abi-entries`), (`step:wave0:bite-0-5-native-routes`) and (`step:wave0:bite-0-5-residual-routes`) are minted in the strategy at the base this study reads, `0.7.32-dev`. Every figure is recounted from the tree at that base, and code is cited by path and line at that base.

## S1. The ABI entries · `sec:w0route:abi-entries`

**Fact (What the two entries owe at this base)** · `fact:w0route:entry-standing`

The bounded consolidation (`step:wave0:bite-0-5-close-safety`) left 36 first-party comparisons owed; eight were compared by (`step:wave0:bite-0-5-first-party-entries`), leaving 28 at this base (`rec:backlog:wave0-first-party-entries`), fifteen of them §16.9 classification rows and six §16.11 role rows whose owning entries are named but absent (`rec:backlog:wave0-close-safety`), (`tab:backlog:safety-first-party-rows`). Their entry strings read `ABI classification: required transaction::validate_maturity_abi_classification` and `ABI role validation: required transaction::validate_maturity_abi_roles` (`packages/vectors/src/maturity_first_party.rs:4393-4398`), and `retained_obligations_have_valid_subjects_and_named_boundaries` pins 28 retained rows, 15 classification rows and 6 role rows (`packages/vectors/src/maturity_first_party.rs:5479-5481`). The conjunct table holds the matching part of conjunct 14 WAITING (`tab:backlog:conjunct-dispositions`). All 21 rows declare the pre-target layer `AbiConstructionRejection`, the carrier `CoordinatorStructure` and the matrix control `SponsorlessAnnouncement` (`packages/vectors/src/maturity_safety.rs:2620-2761`), (`packages/vectors/src/maturity_safety.rs:2925-2983`), (`packages/vectors/src/maturity_safety.rs:1398-1413`). Guide 13's policy therefore governs them: each needs a canonical malformed typed input refused by its exact owning first-party validator with a typed refusal naming the intended class, beside a focused accepted control, and a refusal from another layer answers nothing (`rule:guide13-exec:first-party-negatives`); a pre-target class is never submitted to a target merely because an executor exists (`rule:guide13-exec:abi-validation-role`).

**Specification (Where the entries live)** · `spec:w0route:entry-home`

Both entries belong in `packages/transaction/src/state_abi.rs`, beside `CandidateMaturityAnnouncementAbi` (`packages/transaction/src/state_abi.rs:639-652`) and `derive_maturity_announcement_abi` (`packages/transaction/src/state_abi.rs:761-797`), for the reason the compact precedent sits where it does: `validate_compact_abi_fields`, its refusal `CompactAbiValidationRefusal` and its result `ValidatedCompactAbiFields` live beside the compact ABI they validate (`packages/transaction/src/abi.rs:55-127`), and the transaction package owns the final typed ABI (`tab:abi:ownership`). The compact entry checks fields of an existing transaction and states that it admits nothing to safe construction (`packages/transaction/src/abi.rs:89-93`); the maturity entries keep that standing. Neither changes `construct_maturity_announcement_with_sponsor` or any existing production validator: both are new entries with their own refusal and result types, re-exported from `packages/transaction/src/lib.rs`. A sibling module would be justified only if the two entries outgrew the derivation they validate; the precedent favours co-location.

**Specification (The classification entry)** · `spec:w0route:classification-entry`

The entry takes what is offered — serialized target bytes, the complete spent-output census and one claimed role per input and output position — together with the ABI, the validated view and the scoped realization that owns the relations (`packages/realization/src/derive.rs:70`):

```rust
pub fn validate_maturity_abi_classification(
    abi: &CandidateMaturityAnnouncementAbi,
    view: &ValidatedMaturityStateView,
    realization: &realization::ScopedRealizationSpec,
    offer: &MaturityAbiOffer<'_>,
) -> Result<ValidatedMaturityAbiClassification, MaturityAbiClassificationRefusal>;

pub struct MaturityAbiOffer<'a> {
    pub bytes: &'a [u8],
    pub spent: &'a [SpentOutputCensusEntry],
    pub reserve_asset: AssetId,
    pub inputs: Vec<MaturityPositionClaim>,
    pub outputs: Vec<MaturityPositionClaim>,
    pub canonical: realization::ObservedCanonicalPartition,
}

pub enum MaturityPositionClaim {
    State,
    SponsorMember,
    SponsorChange,
    TargetFee,
    Family(architecture::ObjectId),
    Projection(architecture::ProjectionId),
    Destruction,
    DataOutput(Option<architecture::TagId>),
}

pub enum MaturityAbiClassificationRefusal {
    Decode(Box<TransactionRefusal>),
    IssuanceOffered,
    SpentCensus { inputs: usize, entries: usize },
    Unclassified { side: realization::ObservedSide, position: usize },
    ClaimContradictsBytes { side: realization::ObservedSide, position: usize },
    UnadmittedOutputRole { position: usize, claim: MaturityPositionClaim },
    RelationVerdicts {
        failed: BTreeMap<realization::RelationId, realization::RelationFailure>,
    },
    Realization(Box<realization::RealizationError>),
}
```

Bytes rather than a decoded `TargetTransaction` are offered because the decoder refuses an issuance flag before any typed value exists (`packages/transaction/src/bytes.rs:709-719`): a decoded signature could never be offered the `add-issuance` subject, so the entry decodes as its own first stage and maps the decoder's `IssuanceInputRefused` to `IssuanceOffered` without a second reading of the flag. In order, the entry (1) decodes; (2) requires one census entry per input, in position order; (3) requires exactly one claim per position on each side, which is the coverage and disjointness of (`rule:abi:ranges`); (4) authenticates every admitted-role claim against the bytes — a STATE claim carries the view's asset and singleton amount, a sponsor claim the declared reserve asset, the fee claim an empty program with an explicit value — and refuses a contradicted claim; (5) refuses an output claimed as a destruction or a data output, because the three output roles admit neither (`packages/transaction/src/state_abi.rs:212-259`); (6) projects the remaining claims into one `realization::OperationObservation` with sponsor values opaque, the projection the construction tests already build by hand (`packages/transaction/src/tests/state_construct_tests.rs:271-358`); and (7) evaluates it once through the realization's evaluator (`packages/realization/src/derive.rs:183-198`), refusing with the failed verdicts of exactly the relations it owns: both `AllowedObjectFamilies` closures, `OpenFlowPolicy`, `CanonicalDeltaPolicy` and `ProjectionPolicy` (`packages/realization/src/declarations/announce_maturity.rs:256-271`), (`packages/realization/src/declarations/announce_maturity.rs:322-349`). The validated result retains the decoded transaction, the claims and the evaluated report, so the role entry reads that evaluation rather than repeating it. The entry enforces the families of (`rule:abi:semantic-layout`) and the coverage, disjointness and complete protocol-family classification of (`rule:abi:ranges`), including that no output region stays unclassified.

**Specification (The role entry)** · `spec:w0route:role-entry`

The role entry takes the validated classification as evidence that every position is claimed and its family admitted:

```rust
pub fn validate_maturity_abi_roles(
    abi: &CandidateMaturityAnnouncementAbi,
    view: &ValidatedMaturityStateView,
    classified: &ValidatedMaturityAbiClassification,
    sponsor_input_maximum: Option<realization::Count>,
) -> Result<ValidatedMaturityAbiRoles, MaturityAbiRoleRefusal>;

pub enum MaturityAbiRoleRefusal {
    CoordinatorMissing,
    CoordinatorIsNotTheValidatedPredecessor { expected: Outpoint, offered: Outpoint },
    CoordinatorRepeated { positions: Vec<usize> },
    StateAndSponsorPositionsExchanged { state: usize, sponsor: usize },
    Cardinality {
        relation: realization::RelationId,
        failure: realization::RelationFailure,
    },
    SuccessorDisplaced { expected: u16, offered: usize },
    SponsorSuffixDisplaced { first: u16, offered: usize },
    SponsorChangeDisplaced { offered: usize },
    FeeNotFinal { offered: usize },
}
```

The checks run in a fixed order so each row's subject meets its own check first. First the coordinator rule of (`rule:abi:coordinator`): the input at the stated coordinator index (`packages/transaction/src/state_abi.rs:144-155`), (`packages/transaction/src/state_abi.rs:183-197`) carries a STATE claim, spends the view's current STATE outpoint, and that outpoint appears once; a STATE claim found elsewhere with a sponsor member at the coordinator index is the exchanged-positions refusal, and no STATE claim at all is the missing-coordinator refusal. Then the STATE input and output `Cardinality` verdicts, read from the classification's retained report (`packages/realization/src/declarations/announce_maturity.rs:163-184`). Then the stated positions of the successor, the sponsor suffix, the sponsor change and the fee (`packages/transaction/src/state_abi.rs:157-176`), (`packages/transaction/src/state_abi.rs:261-299`), which with the cardinalities are the census and position halves of (`rule:abi:coordinator`) and (`rule:abi:ranges`). The sponsor input cardinality is bounded by `FeeSponsorInputMax`, which no calibration in the transaction package supplies, and the ABI deliberately states only where the suffix begins (`packages/transaction/src/state_abi.rs:157-164`). The entry therefore takes that maximum as an option: with `None`, the projection supplies the offered sponsor count so evaluation can run, and the validated result carries the residual `SponsorInputMaximumUncalibrated`, so no reader can mistake that verdict for a check. None of the six rows depends on it.

**Argument (Offered claims are proposals and the refusal does not rest on them)** · `arg:w0route:claims-not-trusted`

ADR-006 treats counts as untrusted proposals and requires every region that could carry a closed asset to be classified (`rule:abi:ranges`), and the realization takes region membership from the observation's producer (`packages/realization/src/observation.rs:210-219`). The offer's claims are that producer's statement, authenticated wherever the deployment fixes the answer: a claim of STATE, sponsor member, sponsor change or fee is checked against asset, amount and program at its position, and every other claim names a family the announcement does not admit. The verdict on a subject is therefore claim-independent — an added closed-asset output claimed as a sponsor member contradicts the reserve asset, claimed as STATE it contradicts the STATE asset or the cardinality, left unclaimed it is unclassified, and claimed truthfully it fails a family closure — while the class the refusal names depends on the honest claim. Each comparison pins both facts: the exact refusal under the honest claim, and a refusal under every alternative claim of the same bytes. The transaction package never infers another family from bytes: it holds no census of other families' programs, and recognizing them would be a second statement of facts the architecture and the linker own (`sec:abi:limits`).

**Argument (Relations are read from the realization, never restated)** · `arg:w0route:relations-read`

The allowed families, the empty canonical-delta set, the forbidden projections, the admitted open-flow kind and the STATE cardinalities are declared once, in the announcement's declarations (`packages/realization/src/declarations/announce_maturity.rs:163-206`), (`packages/realization/src/declarations/announce_maturity.rs:249-368`), and evaluated once, by the realization (`packages/realization/src/evaluate.rs:387-420`), (`packages/realization/src/evaluate.rs:480-495`), (`packages/realization/src/evaluate.rs:707-753`). The entries never copy a family set, a projection rule or a bound; they project positions into an observation and read verdicts by relation identity, as `sponsor_status` already does in the construction tests (`packages/transaction/src/tests/state_construct_tests.rs:361-383`). `SponsorIsolation` and `SponsorEnvelopeMultiplicity` are evaluated in the same pass but are not ABI-entry refusals: their sponsor rows are already answered by landed construction and region evidence (`rec:backlog:wave0-sponsored-retyping`), and reporting them here would open a second answer channel for those rows. Rows the matrix leaves unlinked keep an ABI-layout refusal rather than borrowing a relation: issuance, destruction, burn records and clear events are not a named canonical asset-flow family (`packages/vectors/src/maturity_safety.rs:5799-5806`), so their refusals are `IssuanceOffered` and `UnadmittedOutputRole`, and the relation partition stays 14 / 108 / 84.

**Specification (The typed result under guide 13's three-way distinction)** · `spec:w0route:three-way-result`

The transaction entries return a validated value or a typed refusal; the three-way record lives in the vectors package and reuses `AbiValidationOutcome` rather than minting a second vocabulary (`packages/vectors/src/abi_validation.rs:173-196`). Per row, `AbiFieldRefused` — guide 13's safe constructor refused — holds when the transaction entry accepts the row's control and returns the exact refusal for every staged subject; `UnsafeRawMutationExists` holds when a staged subject's bytes exist only by test surgery and the entry does not refuse them, which is a finding against the entry, never a disposition; `TargetWasNotAsked` is the honest standing of every row before its entry lands, which is all 21 rows at this base. The safe constructor's inability to emit these subjects stays structural, because the request has no field for positions, coordinator or families (`packages/transaction/src/state_request.rs:91-158`), and each subject is cut by explicitly scoped test staging, the precedent of `CompactAbiTestStaging` (`packages/vectors/src/abi_validation.rs:84-157`).

**Table (Fifteen classification rows and their subjects)** · `tab:w0route:classification-subjects`

Every subject departs from the row's sponsorless control by the one coordinate named, with the census entry of an added input. A row naming two families or two sides carries one subject per family or side and is discharged only when each subject refuses: 21 subjects across 15 rows. "Output closure" and "input closure" are the side-specific `AllowedObjectFamilies` relations.

| Key | Accepted control | Changed coordinate | Expected typed refusal under the honest claim |
|---|---|---|---|
| `16.9 / add-live-receipt-input-or-output` | Sponsorless announcement | (i) output 1 added with the live-receipt family's asset, claimed `Family(ReceiptLive)`; (ii) input 1 added likewise | (i) `RelationVerdicts` {output closure: `UndeclaredObjectFamily`}; (ii) the same on the input closure |
| `16.9 / add-time-locked-receipt` | Sponsorless announcement | Output 1 added, claimed `Family(ReceiptTimeLocked)` | `RelationVerdicts` {output closure: `UndeclaredObjectFamily`} |
| `16.9 / add-ash` | Sponsorless announcement | Output 1 added, claimed `Family(Ash)` | `RelationVerdicts` {output closure: `UndeclaredObjectFamily`} |
| `16.9 / add-entitlement` | Sponsorless announcement | Output 1 added, claimed `Family(DepositEntitlement)` | `RelationVerdicts` {output closure: `UndeclaredObjectFamily`} |
| `16.9 / add-request` | Sponsorless announcement | Output 1 added, claimed `Family(DepositRequest)` | `RelationVerdicts` {output closure: `UndeclaredObjectFamily`} |
| `16.9 / add-distribution-control-or-vault` | Sponsorless announcement | (i) output 1 claimed `Family(DistributionControl)`; (ii) output 1 claimed `Family(DistributionVault)` | `RelationVerdicts` {output closure: `UndeclaredObjectFamily`} for each |
| `16.9 / add-resv-input-or-output` | Sponsorless announcement | (i) input 1 claimed `Family(Resv)`; (ii) output 1 claimed `Family(Resv)` | (i) input closure; (ii) output closure; `UndeclaredObjectFamily` for each |
| `16.9 / add-pace` | Sponsorless announcement | Output 1 added, claimed `Family(Pace)` | `RelationVerdicts` {output closure: `UndeclaredObjectFamily`} |
| `16.9 / add-authority-object` | Sponsorless announcement | (i) output 1 claimed `Family(EntitlementAuthority)`; (ii) output 1 claimed `Family(DistributionAuthority)` | `RelationVerdicts` {output closure: `UndeclaredObjectFamily`} for each |
| `16.9 / add-issuance` | Sponsorless announcement | Input 0 serialized with the issuance flag and a well-formed issuance of a fresh asset | `IssuanceOffered` |
| `16.9 / add-destruction` | Sponsorless announcement | Output 1 added: the unspendable data-carrier program with a nonzero explicit value of a closed asset, claimed `Destruction` | `UnadmittedOutputRole { position: 1, claim: Destruction }` |
| `16.9 / add-burn-record` | Sponsorless announcement | Output 1 added: a zero-value null-data output carrying the `tag-burn` bytes, claimed `DataOutput(Some(Burn))` | `UnadmittedOutputRole { position: 1, claim: DataOutput(Some(Burn)) }` |
| `16.9 / add-clear-event` | Sponsorless announcement | Output 1 added: a zero-value null-data output carrying the clear tag, claimed `DataOutput(Some(Recon))` | `UnadmittedOutputRole { position: 1, claim: DataOutput(Some(Recon)) }` |
| `16.9 / add-residue-projection` | Sponsorless announcement | Output 1 added: a null-data output carrying `tag-distribution-residue`, claimed `Projection(DistributionResidue)` | `RelationVerdicts` {projection policy: `ProjectionPolicy`}, the matrix's `ForbiddenProjectionPresent` |
| `16.9 / introduce-canonical-u-ent-or-dist-ctl-flow` | Sponsorless announcement | (i) to (iii) for `U`, `ENT` and `DIST_CTL`: input 1 and output 1 of that asset, each claimed as a family carrying it, with one claimed lateral canonical flow from input 1 to output 1 | `RelationVerdicts` {canonical delta: `CanonicalDeltaPolicy`, the matrix's `UnexpectedCanonicalDeltaFamily`}, with both family closures as declared collateral |

No family the announcement admits carries `U`, `ENT` or `DIST_CTL`, so the canonical-flow subject cannot be isolated from the family closures; it is distinguished by its verdict set, which contains the canonical-delta failure the row names, and its collateral is declared in its staging as native staging declares named collateral (`rec:backlog:wave0-safety-native-staging`).

**Table (Six role rows and their subjects)** · `tab:w0route:role-subjects`

Each subject passes the classification entry, so its refusal comes from the role entry.

| Key | Accepted control | Changed coordinate | Expected typed refusal |
|---|---|---|---|
| `16.11 / wrong-coordinator` | Sponsorless announcement | Input 0 spends another STATE-asset coin at a STATE program, the stand-in second coin native staging uses, with its census entry | `CoordinatorIsNotTheValidatedPredecessor { expected, offered }` |
| `16.11 / duplicate-coordinator` | Sponsorless announcement | Input 1 added, spending the view's predecessor outpoint again with the same census entry, claimed `State` | `CoordinatorRepeated { positions: [0, 1] }` |
| `16.11 / no-coordinator` | Sponsorless announcement | Input 0 replaced by a reserve-asset coin claimed `SponsorMember` | `CoordinatorMissing` |
| `16.11 / state-and-sponsor-positions-exchanged` | Sponsored S1 construction, under the control reading of (`tab:w0route:ambiguities`) | Inputs 0 and 1 exchanged with their census entries and claims | `StateAndSponsorPositionsExchanged { state: 1, sponsor: 0 }` |
| `16.11 / wrong-input-count` | Sponsorless announcement | Input 1 added: a distinct STATE-asset coin at a STATE program, claimed `State` | `Cardinality { relation: input STATE cardinality, failure: CardinalityAboveMaximum }` |
| `16.11 / wrong-output-count` | Sponsorless announcement | Output 1 added: the STATE asset and singleton amount at a STATE program, claimed `State` | `Cardinality { relation: output STATE cardinality, failure: CardinalityAboveMaximum }` |

**Specification (The typed request-cycle decoder)** · `spec:w0route:cycle-decoder`

`16.2 / malformed-cycle-encoding` declares `SemanticRequestRejection` at the request field (`packages/vectors/src/maturity_safety.rs:1657-1665`) and is retained because the typed request admits no encoding at all (`packages/vectors/src/maturity_first_party.rs:3089-3092`). The encoding exists: the requested cycle is witness item 2, written as the eight big-endian bytes of `Cycle::get` (`packages/transaction/src/state_signing.rs:1020-1025`) and read back by the readers that decode the witnessed request. The proposed boundary in `packages/transaction/src/state_request.rs` is its inverse:

```rust
impl MaturityAnnouncementRequest {
    pub fn decode_announced_cycle(
        abi: &CandidateMaturityAnnouncementAbi,
        encoded: &[u8],
    ) -> Result<Cycle, MaturityRequestDecodingRefusal>;
}

pub enum MaturityRequestDecodingRefusal {
    CycleWidth { declared: usize, offered: usize },
}
```

The width is read from the ABI's `RequestedCycle` witness role, whose bounds come from the composed record (`packages/transaction/src/state_abi.rs:888-929`), not restated. The accepted control is the encoding the signing stage writes for the retained subject's request, which decodes to that request's cycle; the malformed offer drops the final byte; the refusal is `CycleWidth { declared: 8, offered: 7 }`. `Cycle` wraps a `u64` (`packages/realization/src/domain.rs:141-169`), so every eight-byte string is a cycle and width is the only malformed coordinate unless the lead-window fragment reads the item over a narrower domain, which the step checks (`tab:w0route:ambiguities`). The decoder has production consumers — the continuity and recovery readers that decode witness item 2 — so it is not a test-only entry, and adding it broadens nothing.

**Argument (The noncanonical nonce has an offerable coordinate)** · `arg:w0route:nonce-offerable`

`16.3 / correct-semantic-metadata-with-noncanonical-representation-nonce` carries `LeastnessIsAPropertyOfTheSearchNotOfAnOffer` (`packages/vectors/src/maturity_first_party.rs:3107-3111`), and its entry string names a canonical search comparison at `CandidateStateConstructor::derive` (`packages/vectors/src/maturity_first_party.rs:4389`). The reading that no coordinate is offerable does not hold at this base. `state_output_program_at_nonce` exists so that a party holding a published program can check it against the metadata and nonce it was given (`packages/tapscript/src/state_constructor.rs:1125-1152`); an observed announcement offers the successor nonce as witness item 1; the continuity reader already runs a separate host search and records whether the witnessed nonce is host-least without refusing (`packages/vectors/src/maturity_continuity.rs:837-875`); and the native observation offered exactly this coordinate, the next admissible nonce, which the target accepted (`tab:backlog:native-observation-rows`). An offerable coordinate exists, so the row may not be retired by an absence-of-coordinate argument (`def:guide15-exec:retired-residual`), and the comparison should be built at its owner. Left unbuilt, the row has no standing the gate admits: it is observable, so neither a final reason nor retirement is available, and it would stay first-party-required and hold the first-party gate open (`gate:guide15-exec:bite-0-5-first-party-gate`).

**Specification (The validator-side nonce comparison)** · `spec:w0route:nonce-comparison`

```rust
impl CandidateStateConstructor {
    pub fn compare_offered_representation(
        target: &ReviewedElementsTapscriptDefinition,
        offered: &EncodedStateMetadata,
        static_subtree: &StateStaticSubtree,
        policy: StateInternalKeyPolicy,
        budget: StateNonceBudget,
        curve: &impl StateCurveCapability,
    ) -> Result<StateNonceEvidence, StateConstructorRefusal>;
}
```

The comparison first commits the offered encoding through the one commitment both existing callers share (`packages/tapscript/src/state_constructor.rs:797-834`), so an inadmissible offered nonce refuses with that attempt's own refusal and belongs to another row; it then runs `derive` over the offered semantic (`packages/tapscript/src/state_constructor.rs:865-911`) and compares the selected nonce with the offered one. Equality returns the search's `StateNonceEvidence`; a smaller selected nonce refuses with a new non-retryable `StateConstructorRefusal::NoncanonicalRepresentationNonce { offered, least }`; exhaustion before the offered nonce refuses with `OfferedRepresentationBeyondSearchBudget`, keeping the stated residual that leastness is host leastness within the budget (`packages/tapscript/src/state_constructor.rs:773-786`). The accepted control is the successor's own selected nonce; the malformed offer is the same semantic at its next admissible nonce, the encoding the native `LaterAdmissibleNonce` subject carried; the refusal is `NoncanonicalRepresentationNonce`. The leaf is unchanged: this is a first-party owner check, not a covenant clause, and `16.6 / later-admissible-nonce-instead-of-first` keeps its final undecided-coordinate reason.

**Specification (Files, tests and pins)** · `spec:w0route:abi-files`

Production files: `packages/transaction/src/state_abi.rs` for the offer, the claim, both entries, both refusals and both validated results; `packages/transaction/src/state_request.rs` for the decoder and its refusal; `packages/transaction/src/lib.rs` for exports; `packages/tapscript/src/state_constructor.rs` for the comparison and its two refusal variants, both non-retryable. Transaction tests, in a new `packages/transaction/src/tests/state_abi_validation_tests.rs`: both entries accept the sponsorless and S1 controls; each of the 21 classification and 6 role subjects refuses with its exact variant; a claim-independence test offers every alternative claim for each subject's bytes and requires a refusal; the decoder round-trips the signing stage's encoding and refuses the truncation; the construction entry and its refusal census are unchanged. Tapscript tests pin the control, the next-admissible offer and exhaustion. Vectors: four test-staging types in `packages/vectors/src/maturity_first_party.rs` — `MaturityAbiClassificationTestStaging`, `MaturityAbiRoleTestStaging`, `MaturityCycleDecodingTestStaging` and `MaturityNonceComparisonTestStaging` — joined to `MaturityRetainedComparison` (`packages/vectors/src/maturity_first_party.rs:4098-4117`) and to `supplemental_discharge` (`packages/vectors/src/maturity_first_party.rs:4119-4163`), and an index reusing `AbiValidationOutcome`. Pins that move: `supplementary_safety_keys_and_census_are_exact` (`packages/vectors/src/maturity_first_party.rs:5163-5211`), `retained_obligations_have_valid_subjects_and_named_boundaries` (`packages/vectors/src/maturity_first_party.rs:5436-5482`), `supplementary_comparisons_retain_exact_objects_and_refusals`, `supplementary_dispositions_join_required_keys_without_promoting_historical_answers`, `closure_categories_partition_without_turning_finality_into_observation`, `closure_category_publication_is_exact_and_refuses_each_changed_count` and `safety_disposition_record_matches_the_typed_ledger`; the entry strings lose their `required` prefix once the entries exist (`packages/vectors/src/maturity_first_party.rs:4385-4411`). No relation, matrix boundary, fixture or archive changes, and the Safety report schema stays at 3 because no key is added.

**Calculation (Census movement of the ABI-entries step)** · `calc:w0route:abi-census`

At the bounded consolidation (`step:wave0:bite-0-5-close-safety`), the census was 206 / 139 / 67 — observed 139, final 22, retired 2, waiting 43 — with first-party 45 discharged and 36 required (`rec:backlog:wave0-close-safety`). The first-party-entries step (`step:wave0:bite-0-5-first-party-entries`) compared eight rows at existing entries, so at this base the census is 206 / 147 / 59 — observed 147, final 22, retired 2, waiting 35 — with first-party 53 discharged and 28 required (`rec:backlog:wave0-first-party-entries`), (`result:backlog:first-party-entries`). The 28 required are 15 classification rows, 6 role rows, 1 cycle-decoder row, 1 nonce-comparison row and 5 adapter request rows (`tab:backlog:safety-first-party-rows`); waiting is those 28 first-party obligations plus seven typed waits. The ABI-entries step then discharges the 22 rows whose entries it builds: first-party required 28 → 6 and discharged 53 → 75, census 206 / 169 / 37, observed 169, final 22, retired 2, waiting 13 = 6 + 7, and 169 + 22 + 2 + 13 = 206. The nonce row is one of the six the first-party-entries row retains; built here, it moves first-party required 28 → 5, discharged to 76, the census to 206 / 170 / 36 and waiting to 12. Typed non-answers stay 29 (22 final, 7 waiting), native acceptance and refusal 11 / 38, report-layer 0 required / 15 observed, and relations 14 / 108 / 84 with reasons 55 / 9 / 9 / 0 / 11. The conjunct-14 part for ABI classification and roles moves WAITING → MET only when all 21 rows compare.

**Proposal (Record shape, size and dependencies)** · `proposal:w0route:abi-record`

A new backlog record after (`rec:backlog:wave0-close-safety`), headed ``rec:backlog:wave0-abi-entries``, carries one table in the form of (`tab:backlog:report-consumer-readings`) — key, accepted control, changed coordinate, owning entry and exact refusal, standing after, pinning test — with 22 or 23 rows, a census result paragraph, and caveats for claim independence, the uncalibrated sponsor maximum, the ABI-layout refusals of unlinked rows and the unchanged relation links. `T12-005` and the conjunct table update in the same landing. Size: four medium sub-steps (`def:wave0:size-classes`), because each adds one boundary or closes a finite row family and none changes a relation or runs a target. Dependencies: the first-party-entries row, which shares the ledger and staging module; no target and no ruling; the decoder and nonce comparison are independent of the two ABI entries.

**Proposal (Guide amendment for the ABI entries)** · `proposal:w0route:abi-guide-amendment`

The following sentence is applied to (`setup:guide15-exec:bite-0-5-files-and-packages`): "The ABI-entry routes add `transaction::validate_maturity_abi_classification`, `transaction::validate_maturity_abi_roles` and the typed request-cycle decoder in `packages/transaction/src/state_abi.rs` and `packages/transaction/src/state_request.rs`, with exports in `packages/transaction/src/lib.rs` and the three-way outcome in `packages/vectors/src/abi_validation.rs`, and the validator-side nonce comparison in `packages/tapscript/src/state_constructor.rs`, each with its tests; each is a new entry beside its owner and broadens no existing production validator (`pkg:transaction:contract`); (`pkg:tapscript:contract`)."

## S2. The residual routes · `sec:w0route:residual-routes`

### (a) The balanced confidential sponsor route · `sec:w0route:confidential-route`

**Fact (What the STATE route lacks)** · `fact:w0route:confidential-gap`

The explicit construction refuses a committed sponsor input value and a committed change with `MaturitySponsorValueRequiresProofFinalization` (`packages/transaction/src/state_construct.rs:426-428`), (`packages/transaction/src/state_construct.rs:442-444`), builds through `TargetTransaction::new`, which fixes every output witness empty (`packages/transaction/src/bytes.rs:1121-1143`), and finalizes over the witnessless encoding, which for an explicit candidate equals the full one (`packages/transaction/src/state_finalize.rs:64-85`); its signing chain states that it carries no confidential proofs (`packages/transaction/src/state_signing.rs:18-23`). The live lane's materializer has every stage this route lacks: deterministic blinders, the balancing solve, value commitments checked against an independent origin, nonce commitments, range proofs with an empty surjection field, the output-witness vector, and a freeze whose protected preimage includes the output witnesses (`packages/transaction/src/live_materialize.rs:1999-2018`), (`packages/transaction/src/live_materialize.rs:1813-1838`). The row `16.10 / confidential-sponsor-value-where-the-selected-target-policy-claims-support` waits under `ConfidentialSponsorAwaitsBalancedProofRoute` (`packages/vectors/src/maturity_safety.rs:313-319`), (`packages/vectors/src/maturity_safety.rs:2908-2916`).

**Argument (The live traits serve the STATE form; the live materializer does not)** · `arg:w0route:live-traits`

In an announcement the protocol region is explicit — the successor's asset and singleton amount are the deployment's own definitions, which the leaf's structural patterns pin (`packages/transaction/src/state_construct.rs:483-537`) — and the fee is mandatorily explicit, so a committed sponsor input's blinder has exactly one committed output to land on: the sponsor change. Its blinder is forced, the solve over an empty set of other blinders, which returns the input blinder sum (`packages/transaction/src/live_materialize.rs:516-523`), (`packages/transaction/src/live_materialize.rs:391-402`). The two injected traits supply exactly that: `ConfidentialProofMaterializer` for the change's value commitment, nonce field and range proof, and `IndependentCommitmentCheck` for the recomputation and the solve (`packages/transaction/src/live_materialize.rs:343-403`). `materialize_confidential_candidate` cannot be reused without broadening it: its balancing output is a protocol-region member and its sponsor change brings a freely chosen blinder (`packages/transaction/src/live_materialize.rs:639-664`), the opposite of what the announcement needs, and admitting a solved sponsor change there would widen the live lane's admitted shapes. The proposal is a STATE-local stage, a new `packages/transaction/src/state_confidential.rs`, that calls the two traits with its single election, emits the output-witness vector through `TargetTransaction::with_output_witnesses` (`packages/transaction/src/bytes.rs:1161-1195`), and hands a proof-finalized candidate to a proof-finalized form of `FinalizedMaturityAnnouncement` whose protected comparison covers output witnesses but not input witnesses; the finalized-fact census already names output witnesses (`packages/transaction/src/state_finalize.rs:132-171`). Operator and sponsor signing then bind the proofs, as the live freeze's preimage does. No existing validator widens: the explicit route's refusal stays, pinned by `confidential_sponsor_value_requires_balanced_proof_route`.

**Fact (What the target form refuses, and why the refusal is a form limit)** · `fact:w0route:form-refusal`

The decoder admits the hybrid form — explicit asset, committed value, a required range proof and an empty surjection field — and refuses a surjection proof unconditionally, a range proof on an explicit value, a committed value without a range proof, and issuance or peg-in data (`packages/transaction/src/bytes.rs:23-34`), (`packages/transaction/src/bytes.rs:866-908`), (`packages/transaction/src/bytes.rs:963-979`). The announcement's refusal of proofs therefore lives in the explicit construction route, not in `bytes.rs`, and it is a form limitation rather than a model fact: the realization erases sponsor amounts structurally (`packages/realization/src/observation.rs:51-71`), so a committed sponsor value changes no relation's input. Recognition of a sponsor member does need its explicit asset (`packages/realization/src/declarations/announce_maturity.rs:228-245`), which is why the unconditional surjection refusal agrees with the model for the blinded-asset form, a form this route does not build.

**Specification (The selected-policy observation)** · `spec:w0route:confidential-observation`

The adapter's confidential arm already funds sponsor coins whose values are committed and whose asset stays explicit, from a catalogued fixture handle, and records the committed value field for sponsor signing (`scripts/elements-native-executor.py:876-930`), (`scripts/elements-native-executor.py:4451-4602`), (`scripts/elements-native-executor.py:4604-4716`); `submit_maturity` passes raw bytes and records relay, ordinary selection and direct-block routes under node-default policy (`scripts/elements-native-executor.py:4805-4999`). The observation needs no new adapter arm: it needs a fixture registration for the announcement sponsor's coin beside `sponsor_reserve_handle` (`packages/vectors/src/confidential_sponsor_reserve.rs`), a confidential S2 family in the planner's sponsored module (`packages/vectors/src/maturity_native.rs:1761-1846`), and a manifest under (`alg:wave0:campaign-manifest`) declaring `AcceptedTransaction` as the expected boundary, since sponsor erasure leaves the protocol indifferent to a committed sponsor value and the selected sponsor profile admits one. The accepted explicit S2 control precedes the confidential subject in the same capture (`pre:wave0:accepted-control`), and admission follows (`proto:wave0:native-admission`): relay acceptance, assembly membership, ordinary mining and exact readback bind the submitted bytes, the change commitment and its range proof. A refusal is retained with its route as an unexpected-boundary contradiction, never as an answer. The archive is a new four-file directory, `packages/vectors/fixtures/maturity-confidential-run-of-record/`, with manifest, report and ceremony pins and an admission function beside `maturity_sponsored_run_of_record`, or a partition of the shared ceremony of (`spec:w0route:native-capture`); retained evidence belongs under `evidence/wave0/confidential/` (`conv:wave0:artifact-homes`). Production files: `packages/transaction/src/state_confidential.rs`, `state_construct.rs`, `state_finalize.rs`, `state_signing.rs` and `lib.rs`; the fixture registry in `packages/target-elements-conformance/src/`; `packages/vectors/src/maturity_native.rs`, `maturity_corpus.rs` and `maturity_evidence.rs`. Tests: commitments agree across origins, the change absorbs the input blinder, an output-witness mutation after finalization refuses, operator signing binds the proofs, and admission binds bytes, proofs, policy and readback. Census on acceptance: typed non-answers 29 → 28, waiting 7 → 6, answered +1, native acceptance 11 → 12. Size: large (`def:wave0:size-classes`): a construction stage, a new observation and an archive. Dependencies: the sponsored construction and ruling 130's sponsored arm (`rec:backlog:wave0-sponsored-ruling`), the fixture registry and the pinned target.

**Judgment (Wave placement of the confidential route)** · `judg:w0route:confidential-wave`

The route is Wave 0 work under the gate's wording. The row is a waiting typed non-answer, and the non-answer gate discharges waiting rows through observation (`gate:guide15-exec:bite-0-5-non-answer-gate`); its observable remainder may not be transferred (`rule:guide15-exec:observable-retention`); and the sponsored ruling's caveat holds the balanced form and its proofs owed (`rec:backlog:wave0-sponsored-ruling`). The later-wave reading would place it with Wave 9's sponsor consistency (`task:guide15-exec:wave9-deliverable-8`) and leave the gate's text unchanged. Its cost is that Wave 0 cannot exit while the row waits and Wave 9 cannot open before Wave 0 exits, because new semantics are held until (`gate:guide15-exec:wave0`): with the gate unchanged, that reading holds the bite open indefinitely rather than moving the work.

### (b) The combined-projection transaction · `sec:w0route:combined-route`

**Fact (Two barriers)** · `fact:w0route:combined-barriers`

`16.1 / accepted-target-transaction-with-semantic-constructor-root-and-recovery-projections-all-matching` waits under `AcceptanceAwaitsRelayAdmissibility`, the last row on that reason (`packages/vectors/src/maturity_safety.rs:1581-1585`), (`tab:backlog:native-observation-rows`). Sponsorless bytes carry no fee output and are refused by relay at the fee floor, so their only acceptance route is a direct block, which establishes neither relay acceptance nor template selection (`rec:backlog:wave0-sponsored-observation`). Sponsored bytes are relay-accepted, but the continuity reader refuses any transaction whose input or output count is not one (`packages/vectors/src/maturity_continuity.rs:597-632`), and public recovery reads its witness through the same function (`packages/vectors/src/maturity_recovery.rs:379-429`). Different transactions cannot supply the conjunction (`rec:backlog:wave0-close-safety`).

**Argument (The restriction belongs to the readers, not to the model)** · `arg:w0route:reader-limitation`

The leaf pins one input position and one output position — the STATE predecessor and its successor — and enforces no count (`packages/transaction/src/state_abi.rs:35-46`); the staging ledger records that the covenant reads output zero only (`packages/vectors/src/maturity_mutants/native_staging.rs:122-133`); and the ABI's layouts admit a sponsor suffix from input one and sponsor change and fee outputs (`packages/transaction/src/state_abi.rs:157-299`). The §12 clause the readers mirror is a positional clause, and the one-input one-output restriction is a reader limit inherited from the sponsorless form. A route lifts it without broadening a validator by adding positional readers instead of relaxing the existing one: a `witness_at_layout` in `packages/vectors/src/maturity_continuity.rs` reads the STATE witness at the coordinator index and the successor at the successor position, and accepts only a transaction that carries a `ValidatedMaturityAbiRoles` as evidence that every other position is a sponsor or fee role. The sponsored region comparison already reads admitted sponsored bytes by position against retained funding (`packages/vectors/src/maturity_safety.rs:4396-4470`). The sponsorless readers and their refusals stay unchanged.

**Specification (One exact accepted transaction for four projections)** · `spec:w0route:combined-transaction`

One admitted relay-and-template acceptance — an S1 or S2 positive from the admitted sponsored or native archive, each binding submitted and mined bytes, both identities and the block height (`rec:backlog:wave0-sponsored-observation`), (`rec:backlog:wave0-native-observation`) — answers the row when, over byte-identical input, the ABI classification and roles validate; the semantic projection derives the successor from the decoded predecessor and the cycle decoded by the request decoder; the constructor projection reconstructs both sides at the layout positions with host leastness; the root projection is `StateRootEdge::from_continuity` (`packages/vectors/src/maturity_history.rs:105-129`) bound to a checkpoint built from the archive's readback; and the recovery projection rebuilds the successor from a public handoff of the same bytes and agrees with output zero. The handoff gains the sponsor positions' spent outputs, which are public chain data, so the recovery side can carry the same role evidence. The fold in `packages/vectors/src/maturity_evidence.rs` answers the row only when all five agreements hold over one transaction identity. No capture is needed if an admitted acceptance carries every binding; otherwise one sponsored capture under (`alg:wave0:campaign-manifest`) supplies it. Census: typed non-answers fall by one, waiting by one, answered and native acceptance each rise by one, and no row remains on `AcceptanceAwaitsRelayAdmissibility`. Production files: `packages/vectors/src/maturity_continuity.rs`, `maturity_recovery.rs`, `maturity_evidence.rs`, and `maturity_history.rs` only if the edge needs a positional source. Tests: the positional reader refuses without validated roles, the sponsorless readers' refusals are unchanged, and the fold refuses agreements drawn from two transactions. Size: medium without a capture, large with one. Dependencies: both ABI entries and the request decoder.

**Judgment (Wave placement and the one-or-four question)** · `judg:w0route:combined-wave`

The route is Wave 0 work under the gate's wording: the row is a waiting typed non-answer (`gate:guide15-exec:bite-0-5-non-answer-gate`), and conjunct 7's part for the eight semantic forms names it as the remaining wait (`tab:backlog:conjunct-dispositions`). The accepted-byte study left open whether the row is one row or four (`plans/drafts/accepted-byte-classification-study.md:184`); this design reads it as one conjunction, the matrix's transcription, and keeps the denominator at 206. The four-row reading would let separate reports answer constructor, root and recovery rows, at the cost of moving the denominator and every pinned census figure, which requires the guide's §16 text to move first. The later-wave reading — Wave 12's constructor continuity, root-history edge and public successor recovery (`task:guide15-exec:wave12-deliverable-6`), (`task:guide15-exec:wave12-deliverable-7`), (`task:guide15-exec:wave12-deliverable-8`) — costs what (`judg:w0route:confidential-wave`) states: with the gate unchanged, Wave 0 cannot exit while the row waits.

### (c) The diagnostic recapture · `sec:w0route:diagnostic-recapture`

**Specification (What the executor must record)** · `spec:w0route:recapture-executor`

In the direct-block refusal branch of `submit_maturity`, the executor keeps only the token after `TestBlockValidity failed` — its regular expression stops at the first space — and uses `script_error_in` over the client text to classify the layer without recording the script error (`scripts/elements-native-executor.py:4963-4989`). That is why twenty-two refusal observations carry the bare `mandatory-script-verify-flag-failed` (`rec:backlog:wave0-close-safety`). The change records, per submitted subject, a `block_script_error` field: the payload `script_error_in(error.client_detail, CONSENSUS_SCRIPT_PREFIX)` returns (`scripts/elements-native-executor.py:6625-6667`), published only when it is a key of `FAILURE_CLASS_BY_SCRIPT_ERROR` (`scripts/elements-native-executor.py:1166-1169`) and as `unclassified` otherwise, so the child's text never leaves the adapter, the rule `refused_at_consensus` already states (`scripts/elements-native-executor.py:5001-5029`). Each subject's record binds its family's accepted control and that control's direct-block acceptance in the same capture, beside the relay refusal reason already recorded. On the Rust side, `check_reason` (`packages/vectors/src/maturity_native.rs:1857-1881`) gains a reader of the new field that maps it to the conformance package's existing `ObservedFailureClass` (`packages/target-elements-conformance/src/protocol.rs:3319-3330`), and each row gains an expected class — the check the row names, for instance an `OP_EQUALVERIFY` failure for a metadata reconstruction and an invalid Schnorr signature for the operator rows — compared with the recorded one.

**Specification (Admitting a second native archive)** · `spec:w0route:recapture-admission`

The admitted native archive stays immutable. A new four-file directory, `packages/vectors/fixtures/maturity-diagnostic-run-of-record/`, holds the capture, its timing sidecar, `MANIFEST.sha256` and `RUN-REPORT`. `packages/vectors/src/maturity_corpus.rs` gains `MATURITY_DIAGNOSTIC_MANIFEST_SHA256`, `MATURITY_DIAGNOSTIC_RUN_ADDRESS` and a private `MATURITY_DIAGNOSTIC_CEREMONY_SHA256` beside the native constants (`packages/vectors/src/maturity_corpus.rs:82-88`); `DIAGNOSTIC_FILES` and `DIAGNOSTIC_PINS` in the form of `NATIVE_FILES` and `NATIVE_PINS` (`packages/vectors/src/maturity_corpus.rs:567-600`); an `admit_diagnostic_maturity_corpus` reusing `admit_native_with_replay` (`packages/vectors/src/maturity_corpus.rs:2429-2614`) with a replay that also reads the new field; and a `maturity_diagnostic_run_of_record` held in a `OnceLock` as `maturity_native_run_of_record` is (`packages/vectors/src/maturity_corpus.rs:2621-2627`). Admission keeps the native order: census, manifest, report address, common bindings, fee floors, header, environment, operations, payload, exact replay and derived census. The ceremony-id roster stays `report`: the capture script declares `CEREMONY_IDS='report'` with one expected ceremony (`scripts/live-native-maturity-native-capture.sh:44-48`), and the Rust grammar requires `ceremony-id report` in the capture header and the timing sidecar (`packages/vectors/src/maturity_corpus.rs:1468-1474`), (`packages/vectors/src/maturity_corpus.rs:2697`); archives are told apart by directory and pins, and a new id would move a grammar every admitted archive shares. The fold entry is a supplementary attribution ledger in `packages/vectors/src/maturity_evidence.rs`, derived once and admitted against a recomputation in both directions as the first-party ledger is (`rec:backlog:wave0-close-safety`); it attaches a recorded class to each of the twenty-four claims and changes no standing. Production files: the executor, a diagnostic mode of the native capture script with its test script, `packages/vectors/src/maturity_native.rs`, `maturity_corpus.rs` and `maturity_evidence.rs`. Tests: admission binds bytes, policy and recorded classes; each changed file refuses; the attribution ledger leaves every standing and census figure unchanged.

**Requirement (Constraints the step text places on new captures)** · `req:w0route:recapture-constraints`

The recapture amends no admitted archive. Each recaptured subject's spend-time digest equals its staged `PIN_*` digest, or its replay rebuilds it from the live context with the adaptation named, as the native observation did for the maximum-lead subject (`rec:backlog:wave0-native-observation`). Each refusal is credited only beside its own family's accepted control in the same capture (`pre:wave0:accepted-control`). The node runs at its defaults with no fee override. Only a closed-vocabulary script error is published. A recorded class different from the row's intended check is retained as a finding beside the unchanged standing and changes no standing without a separate record. Infrastructure failure and a refusal at another layer answer nothing (`gate:guide15-exec:bite-0-5-native-gate`). The capture adds no answer: it strengthens the twenty-four claims `16.2 / successor-remains-unannounced` and `successor-becomes-complete`; `16.3 / metadata-from-another-state-object`; `16.4 / empty-signature`, `malformed-signature`, `wrong-operator`, `valid-signature-under-another-key`, `valid-signature-over-another-candidate`, `successor-metadata-changed-after-signing` and `successor-program-changed-after-signing`; `16.5 / wrong-state-asset`, `wrong-singleton-amount`, `wrong-predecessor-program`, `predecessor-metadata-reconstructs-another-program` and `wrong-leaf-version`; `16.6 / successor-from-predecessor-metadata-unchanged`, `wrong-announcement-cycle`, `wrong-representation-nonce`, `wrong-leaf-version`, `arbitrary-caller-supplied-output-program` and `no-state-successor`; and `16.11 / witness-item-reorder`, `control-block-from-another-program` and `target-bytes-changed-after-abi-validation` — 2 + 1 + 7 + 5 + 6 + 3 = 24 claims on twenty-two subjects, because two pairs share one subject (`rec:backlog:wave0-close-safety`).

**Judgment (Wave placement of the recapture)** · `judg:w0route:recapture-wave`

Two readings stand. Under the native gate's literal text the declared boundary is the layer the matrix names, `ScriptPathRejection`, and the twenty-four claims meet it; attribution is then a strengthening outside the predicate, carried into the baseline as the non-claim "no opcode attribution" (`cond:guide15-exec:wave0-exit-condition-9`), and it may wait for a later wave at no cost to Wave 0's exit, with the residual that a credited refusal could have come from another check of the same leaf. Under the record's reading, the close-safety step inherited the finding that the direct-block script error must be retained so that the refusing check is named (`rec:backlog:wave0-native-observation`), and a bare verdict cannot show that a refusal came from the row's check rather than another check of the same layer, which the native gate says answers nothing (`gate:guide15-exec:bite-0-5-native-gate`); the recapture is then Wave 0 work, at the cost of an executor change, one capture and one admission. This study proposes the second reading, because the record already states the obligation and because the capture can share the native-routes ceremony (`spec:w0route:native-capture`), leaving the executor change and the fold as its marginal cost. Size: large. Census and non-answers: none move.

**Table (The residual routes at a glance)** · `tab:w0route:residual-summary`

Census figures assume the ABI-entries step has landed with the nonce comparison (206 / 170 / 36, waiting 12); each route's movement is independent of the others.

| Route | Rows | Census and non-answers | Archive and home | Size | Dependencies | Wave reading proposed |
|---|---|---|---|---|---|---|
| (a) Confidential sponsor | `16.10 / confidential-sponsor-value-where-the-selected-target-policy-claims-support` | Answered +1, typed non-answers −1, waiting −1, native acceptance +1 | `maturity-confidential-run-of-record` or a shared partition; `evidence/wave0/confidential/` | large | Sponsored construction, fixture registry, pinned target | Wave 0; a later wave leaves Wave 0 unexited |
| (b) Combined projection | `16.1 / accepted-target-transaction-with-semantic-constructor-root-and-recovery-projections-all-matching` | Answered +1, typed non-answers −1, waiting −1, native acceptance +1 | Admitted sponsored or native archive; a capture only if a binding is missing | medium, large with a capture | ABI entries and request decoder | Wave 0; the four-row reading moves the denominator |
| (c) Diagnostic recapture | The twenty-four refusal claims | None | `maturity-diagnostic-run-of-record`; `evidence/wave0/diagnostic/` | large | Executor change; may share the native-routes ceremony | Wave 0 proposed; the literal reading permits a later wave |

After (a) and (b) the census is 206 / 172 / 34 — observed 172, final 22, retired 2, waiting 10 = 5 first-party + 5 typed — and 172 + 22 + 2 + 10 = 206; the five remaining typed waits are the native construction routes.

## S3. The five native construction routes · `sec:w0route:native-routes`

**Table (Production coordinates and subject changes)** · `tab:w0route:native-coordinates`

The five rows wait under `NativeSubjectAwaitsConstructionRoute` (`packages/vectors/src/maturity_safety.rs:230-239`) and are unstageable by `NativeUnstageable` (`packages/vectors/src/maturity_mutants/native_staging.rs:96-114`). Each new subject adds a `NativeSubject` variant to the twenty-six-member roster and its `ALL` array with a `PIN_*` digest (`packages/vectors/src/maturity_mutants/native_staging.rs:537-632`), moves its row's standing from unstageable to staged, and adds its family to `FAMILIES` and `RANGES` in `submission_rows` (`packages/vectors/src/maturity_native.rs:1015-1068`).

| Key | Production coordinate that must exist | Subject, control and declared boundary | Planner and adapter changes |
|---|---|---|---|
| `16.4 / stale-operator` | A superseded operator key: a deployment generation whose committed operator key replaced an earlier one, which needs an accepted generation policy admitting an operator-key migration (`case:wave0:generation-preferred`) | The current-generation announcement signed with the superseded key over the honest message; control: the same signed with the current key; `ScriptPathRejection` | A rotation family and the superseded signer in the planner; signing by the existing test signers; no new adapter arm |
| `16.6 / successor-from-wrong-semantic-metadata` | A second deployment whose semantic census differs, so another object's successor no longer reconstructs byte-equal to the control | Output 0 carries the successor derived from the other census at its first admissible nonce, re-signed; control: the honest announcement; `ScriptPathRejection` | A second deployment fixture in the planner; no adapter change |
| `16.6 / wrong-control-recipe` | An offerable successor recipe position: either a two-transition chain in which the successor is spent and its recipe is offered as witness item 8, or a schedule that witnesses the successor recipe | Chain reading: the second spend carries the recipe a wrong successor derivation produces; control: the honest second spend; `ScriptPathRejection`; it must differ by bytes from the inherited `16.5 / wrong-control-block` admission or be recorded as a shared subject with an equality argument (`proposal:wave0:closure-packets`) | A chained family that funds once and announces twice; no adapter arm beyond chained submission |
| `16.6 / two-state-successors` | A balanced two-successor construction: the STATE coin plus a second STATE-asset coin from the retained issuance reserve, two outputs at singleton amounts | Outputs 0 and 1 both STATE successors; control: the honest announcement; declared `ScriptPathRejection`, while the covenant reads output zero only, so an acceptance is likely and is recorded as `16.6 / extra-state-like-output` was | An issuance-reserve funding step using the adapter's existing issuance support (`scripts/elements-native-executor.py:3835-3980`) |
| `16.11 / predecessor-and-successor-metadata-witnesses-exchanged` | A successor metadata witness position: a witness schedule that carries successor metadata, which the linked announcement does not | The two metadata items exchanged under that schedule; control: the honest spend under it; `ScriptPathRejection` | A new schedule in tapscript, the linker and the ABI's witness roles, and therefore a new candidate (`cav:w0route:candidate-change`) |

**Specification (One capture plan)** · `spec:w0route:native-capture`

One ceremony runs under (`alg:wave0:campaign-manifest`): a manifest row per claim names operation, candidate, ABI, target and deployment, node-default policy, constructor, the one-coordinate mutation, declared boundary, control id, funded outpoints and the native fold as consumer; controls run before their negatives (`pre:wave0:accepted-control`); and each subject follows (`proto:wave0:native-admission`) from request validation through readback and boundary classification to its typed consumer. The confidential subject of (`spec:w0route:confidential-observation`) and the twenty-two recaptured subjects of (`req:w0route:recapture-constraints`) may share the ceremony as separate manifest partitions, with a reverse map from every shared observation to its claims (`verif:wave0:campaign-join`); sharing is admissible only when every partition is ready, and otherwise each runs alone. The archive is a new four-file directory admitted like the native one, with retained evidence under `evidence/wave0/native-routes/`. On success the five typed waits leave waiting: each row is answered by its observed refusal or, where the candidate leaves the coordinate undecided, takes the undecided-coordinate final reason; typed non-answers fall by the number answered and waiting falls 5 → 0.

**Caveat (Two routes change the candidate of record)** · `cav:w0route:candidate-change`

The schedule reading of `wrong-control-recipe` and the successor metadata witness of `predecessor-and-successor-metadata-witnesses-exchanged` change the leaf, the linked bundle and the ABI's witness roles, so every subject captured over the present candidate would need a compatibility argument and the final consolidation must follow them (`cond:wave0:baseline-freeze`). The chain reading avoids that change for the recipe row; the metadata-witness row has no chain reading. The conjunct table already retires the announcement-byte recipe coordinate and holds the successor-recipe construction waiting (`tab:backlog:conjunct-dispositions`). The choice between building a position on a new candidate and recording that the admitted candidate offers none should be settled before any capture spends subjects, and the second choice is admissible only as a claim outside the admitted contract under (`def:guide15-exec:non-answer-finality`), never as retirement of an observable part (`def:guide15-exec:retired-residual`).

## S4. Formally labelled sub-steps · `sec:w0route:sub-steps`

**Proposal (Seven sub-steps and their landing order)** · `proposal:w0route:sub-step-order`

Each sub-step is one bounded change or observation with a declared input and a stop predicate (`def:wave0:execution-step`). The ABI-entries row divides into (`step:wave0:bite-0-5-abi-classification-entry`), (`step:wave0:bite-0-5-abi-role-entry`), (`step:wave0:bite-0-5-cycle-decoder-and-nonce-comparison`) and (`step:wave0:bite-0-5-abi-comparisons-and-record`); the residual-routes row divides into (`step:wave0:bite-0-5-confidential-sponsor-route`), (`step:wave0:bite-0-5-combined-projection-route`) and (`step:wave0:bite-0-5-diagnostic-recapture`). After the first-party-entries row, the classification entry lands, then the role entry; the decoder and nonce comparison may land beside them; the comparisons and record follow all three. The combined projection follows the comparisons, because its positional readers consume validated roles. The confidential route's construction may proceed in parallel from the start; its observation, the native-routes capture and the diagnostic recapture run as one ceremony or in sequence once the native-routes row has settled which subjects change the candidate (`cav:w0route:candidate-change`). The predicate of (`step:wave0:bite-0-5-close-safety`) is then re-read over the completed ledger as the bite's last act, and bite 0.7 follows under (`cond:wave0:baseline-freeze`) and (`alg:guide15-exec:wave0-graph`). Proposed named commit units for (`tab:wave0:landing-order`): `feat(transaction): validate maturity ABI classification and roles`; `feat(tapscript): compare an offered representation nonce with the canonical search`; `test(vectors): discharge the ABI-entry Safety rows`; `feat(transaction): construct the balanced confidential sponsored STATE form`; `feat(vectors): read sponsored announcements by ABI position`; `feat(vectors): admit the diagnostic and native-route observations`; `docs(plans): close the residual Safety routes`.

**Listing (Proposed step rows and heads)** · `listing:w0route:proposed-steps`

The seven sub-steps now stand in the strategy's (`tab:wave0:bite-0-5-steps`): (`step:wave0:bite-0-5-abi-classification-entry`), (`step:wave0:bite-0-5-abi-role-entry`), (`step:wave0:bite-0-5-cycle-decoder-and-nonce-comparison`), (`step:wave0:bite-0-5-abi-comparisons-and-record`), (`step:wave0:bite-0-5-confidential-sponsor-route`), (`step:wave0:bite-0-5-combined-projection-route`), (`step:wave0:bite-0-5-diagnostic-recapture`).

**Proposal (Sentences for the guide and the strategy)** · `proposal:w0route:amendment-sentences`

The block below gives the applied sentences, each appended to the paragraph its first line names.

```text
Guide 15, (`setup:guide15-exec:bite-0-5-files-and-packages`), after the sentence of the ABI-entry amendment:
The residual routes also touch `packages/transaction/src/state_confidential.rs`, `packages/transaction/src/state_construct.rs`, `packages/transaction/src/state_finalize.rs` and `packages/transaction/src/state_signing.rs` for the balanced confidential form, the continuity, recovery, corpus and evidence modules in `packages/vectors/src/`, `scripts/elements-native-executor.py` and its capture scripts, and the admitted fixture directories under `packages/vectors/fixtures/`.

Guide 15, (`task:guide15-exec:bite-0-5-first-party`):
The fifteen §16.9 classification rows, the six §16.11 role rows and the §16.2 cycle-encoding row are compared at `transaction::validate_maturity_abi_classification`, `transaction::validate_maturity_abi_roles` and the typed request-cycle decoder, and the §16.3 noncanonical-nonce row at the validator-side comparison of `CandidateStateConstructor` (`rule:guide13-exec:first-party-negatives`).

Strategy, (`setup:wave0:bite-0-5-scope`):
The ABI-entries row divides into four formally labelled sub-steps and the residual-routes row into three; each sub-step follows its parent row in the table, lands in the order the table gives except where its step head permits independent preparation, and closes on its own reading of the guide gate it names.
```

**Table (Labels minted in this study)** · `tab:w0route:minted-labels`

Every label below is minted once, at the head or heading named, in `plans/drafts/wave0-route-study.md`; each is a three-part label of area `w0route` whose kind is catalogued and whose head carries the matching genre, and the table mentions it in double backticks so that it mints nothing a second time. The step labels are cited here and minted in the strategy; the proposed backlog record label is mentioned, not minted. The table holds 48 mints: 12 `spec`, 8 `sec`, 6 `tab`, 5 `arg`, 4 `proposal`, 4 `fact`, 3 `judg`, and one each of `abst`, `calc`, `req`, `cav`, `listing` and `data`.

| Label | Kind | Where it is minted |
|---|---|---|
| ``abst:w0route:scope`` | `abst` | Abstract head before S1 |
| ``sec:w0route:abi-entries`` | `sec` | Heading of S1 |
| ``fact:w0route:entry-standing`` | `fact` | S1 |
| ``spec:w0route:entry-home`` | `spec` | S1 |
| ``spec:w0route:classification-entry`` | `spec` | S1 |
| ``spec:w0route:role-entry`` | `spec` | S1 |
| ``arg:w0route:claims-not-trusted`` | `arg` | S1 |
| ``arg:w0route:relations-read`` | `arg` | S1 |
| ``spec:w0route:three-way-result`` | `spec` | S1 |
| ``tab:w0route:classification-subjects`` | `tab` | S1 |
| ``tab:w0route:role-subjects`` | `tab` | S1 |
| ``spec:w0route:cycle-decoder`` | `spec` | S1 |
| ``arg:w0route:nonce-offerable`` | `arg` | S1 |
| ``spec:w0route:nonce-comparison`` | `spec` | S1 |
| ``spec:w0route:abi-files`` | `spec` | S1 |
| ``calc:w0route:abi-census`` | `calc` | S1 |
| ``proposal:w0route:abi-record`` | `proposal` | S1 |
| ``proposal:w0route:abi-guide-amendment`` | `proposal` | S1 |
| ``sec:w0route:residual-routes`` | `sec` | Heading of S2 |
| ``sec:w0route:confidential-route`` | `sec` | Heading of S2 (a) |
| ``fact:w0route:confidential-gap`` | `fact` | S2 (a) |
| ``arg:w0route:live-traits`` | `arg` | S2 (a) |
| ``fact:w0route:form-refusal`` | `fact` | S2 (a) |
| ``spec:w0route:confidential-observation`` | `spec` | S2 (a) |
| ``judg:w0route:confidential-wave`` | `judg` | S2 (a) |
| ``sec:w0route:combined-route`` | `sec` | Heading of S2 (b) |
| ``fact:w0route:combined-barriers`` | `fact` | S2 (b) |
| ``arg:w0route:reader-limitation`` | `arg` | S2 (b) |
| ``spec:w0route:combined-transaction`` | `spec` | S2 (b) |
| ``judg:w0route:combined-wave`` | `judg` | S2 (b) |
| ``sec:w0route:diagnostic-recapture`` | `sec` | Heading of S2 (c) |
| ``spec:w0route:recapture-executor`` | `spec` | S2 (c) |
| ``spec:w0route:recapture-admission`` | `spec` | S2 (c) |
| ``req:w0route:recapture-constraints`` | `req` | S2 (c) |
| ``judg:w0route:recapture-wave`` | `judg` | S2 (c) |
| ``tab:w0route:residual-summary`` | `tab` | S2 (c) |
| ``sec:w0route:native-routes`` | `sec` | Heading of S3 |
| ``tab:w0route:native-coordinates`` | `tab` | S3 |
| ``spec:w0route:native-capture`` | `spec` | S3 |
| ``cav:w0route:candidate-change`` | `cav` | S3 |
| ``sec:w0route:sub-steps`` | `sec` | Heading of S4 |
| ``proposal:w0route:sub-step-order`` | `proposal` | S4 |
| ``listing:w0route:proposed-steps`` | `listing` | S4 |
| ``proposal:w0route:amendment-sentences`` | `proposal` | S4 |
| ``tab:w0route:minted-labels`` | `tab` | S4 |
| ``sec:w0route:readings-and-walls`` | `sec` | Heading of S5 |
| ``tab:w0route:ambiguities`` | `tab` | S5 |
| ``data:w0route:walls`` | `data` | S5 |

## S5. Ambiguities, readings and walls · `sec:w0route:readings-and-walls`

**Table (Ambiguities and proposed readings)** · `tab:w0route:ambiguities`

Each ambiguity names the readings it admits and the reading this design adopts with its reason.

| Ambiguity | Readings | Proposed reading and reason |
|---|---|---|
| The four bite-0.5 parent step rows | Cite them, or mention them | Cite them; the strategy carries their mints at this base |
| Which six rows the first-party-entries row retains | Settled by the existing-entry record | (`rec:backlog:wave0-first-party-entries`) retains `16.3 / correct-semantic-metadata-with-noncanonical-representation-nonce` and the five §16.12 adapter request rows `blank-request`, `malformed-json`, `oversized-request`, `unknown-request-field` and `unterminated-request`; the nonce needs its canonical-offer contract and the adapter rows need cross-language admission |
| Whether the decoder is another layer for `16.9 / add-issuance` | The decoder as the entry's first stage, or as a separate layer | The entry offers bytes and owns its decoding; otherwise no typed input can carry an issuance while bytes can, and the row could be neither discharged nor retired |
| Burn records and clear events: data-output carriers or forbidden projections | An ABI-layout refusal, or a `ProjectionPolicy` verdict | An ABI-layout refusal, matching their unlinked dispositions (`tab:backlog:relation-dispositions`); a projection reading for the clear event would turn a non-link into a link and belongs to the relation gate |
| The clear event's tag | `tag-recon`, or a tag its future carrier selects | `tag-recon` until bite 0.7's carrier disposition; the refusal does not depend on the tag, because the output layout admits no data-output role |
| The uncalibrated sponsor input maximum | Invent a bound, skip the relation, or carry a residual | Carry `SponsorInputMaximumUncalibrated`; no row of the six depends on the verdict |
| The control of `16.11 / state-and-sponsor-positions-exchanged` | The matrix's sponsorless control, which has no sponsor position, or the S1 control | A per-row S1 control recorded with its reason, as for the two §16.4 rows whose control needs a ruling (`rec:backlog:wave0-sponsored-retyping`); if a per-row control is refused, the row needs a control ruling |
| The cycle's malformed coordinate | Width only, or width and a narrower domain | Width only, unless the lead-window fragment reads the item over a narrower domain, which the decoder step checks |
| Acceptance of a confidential sponsor profile | Ruling 130's arm suffices, or a numbered acceptance is needed | Ruling 130 keeps the confidential form owed under its arm (`rec:backlog:wave0-sponsored-ruling`); whether the profile is a position needing its own acceptance (`req:guide15-exec:position-acceptance`) is to be settled before the confidential construction lands |
| The combined row, one or four | One conjunction, or four rows | One conjunction, keeping the denominator at 206 until the guide's §16 text moves |
| The recapture's wave | Wave 0, or a later wave with a stated non-claim | Wave 0, as argued in (`judg:w0route:recapture-wave`) |
| Native routes that change the candidate | A new candidate, or a scope argument | Settle before any capture, as stated in (`cav:w0route:candidate-change`) |
| This file's place in the drafts census | Listed now, or at integration | The entries in `plans/drafts/meson.build` and (`tab:drafts:index`) were added at the landing, as the drafts census requires |
| Line citations | Lines at `0.7.31-dev`, or lines at the landing base | Lines recounted at `0.7.32-dev`; the landing updates those that moved, as earlier drafts recorded in (`tab:drafts:index`) |

**Data (Walls of the study)** · `data:w0route:walls`

Measured with `date -u +%s.%N`, the authoring reading wall is 1229.95 s, the writing wall is 606.89 s and the total wall is 1836.84 s.
