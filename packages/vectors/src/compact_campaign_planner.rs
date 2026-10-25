// Shared settlement and schedule; malformed assembly is supplied only by tests.
use candidate::bundle::{FixtureBundle, ceremony_bundle, fixture_bundle};
use candidate::compact_campaign::{
    ByteRecipe, CampaignRow, ControlFamily, requirement_name, roster,
};
use candidate::fixture::{SemanticFixtureId, SponsorCase, semantic_case};
use candidate::materialize::{
    AshFunding, SponsorCoin, materialize, materialize_sponsored, sponsor_signing_requests,
    vector_id,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    FundedOutput, NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationSubject,
    TargetFundingSubject, TargetSponsorFundingSubject, TargetSponsorSigningSubject,
    TargetSubmissionSubject, WireOutpoint, WireSighashProfile,
};
use transaction::{
    InputWitness, Outpoint, SponsorSignature, TargetInput, TargetOutput, TargetTransaction, Txid,
};
pub(crate) type Realize =
    Box<dyn FnMut(&[u8], ByteRecipe, Option<Outpoint>, u8) -> Result<Vec<u8>, String>>;
pub(crate) type AdditionalRealize =
    Box<dyn FnMut(&[u8], ByteRecipe, &FundedOutput, u8) -> Result<Vec<u8>, String>>;
pub(crate) fn hex_bytes(bytes: &[u8]) -> String {
    let mut text = String::new();
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}
#[derive(Clone)]
pub(crate) struct Job {
    pub(crate) family: ControlFamily,
    pub(crate) row: Option<CampaignRow>,
}

#[derive(Clone, Copy)]
pub(crate) enum Stage {
    Issue,
    IssueSecond,
    Sponsor,
    Derive,
    Fund,
    Extra,
    SignMutant,
    SubmitMutant,
    SignControl,
    SubmitControl,
    Done,
}

pub(crate) struct Planner {
    realize: Realize,
    additional_realize: Option<AdditionalRealize>,
    pub(crate) second: bool,
    second_asset: Option<[u8; 32]>,
    extra_signer: Option<SponsorCoin>,
    signing_index: usize,
    pub(crate) rows: Vec<CampaignRow>,
    pub(crate) jobs: Vec<Job>,
    pub(crate) stage: Stage,
    pub(crate) index: usize,
    pub(crate) asset: Option<[u8; 32]>,
    pub(crate) bundle: Option<FixtureBundle>,
    pub(crate) program: Vec<u8>,
    pub(crate) sponsors: Vec<SponsorCoin>,
    pub(crate) sponsor_index: usize,
    pub(crate) active_sponsor: Option<SponsorCoin>,
    pub(crate) control: Vec<u8>,
    pub(crate) mutant: Vec<u8>,
    pub(crate) pending: Option<OperationStep>,
    pub(crate) exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    pub(crate) accepted_families: BTreeSet<ControlFamily>,
    pub(crate) suspended: BTreeSet<ControlFamily>,
    pub(crate) findings: Vec<String>,
    pub(crate) failure: Option<String>,
}

pub(crate) fn hex32(text: &str) -> [u8; 32] {
    assert_eq!(text.len(), 64);
    let mut result = [0; 32];
    for (slot, pair) in result.iter_mut().zip(text.as_bytes().as_chunks::<2>().0) {
        *slot = u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII"), 16).expect("hex");
    }
    result
}

pub(crate) fn outpoint(coin: &FundedOutput) -> Outpoint {
    Outpoint::new(
        Txid::from_target_display(&coin.outpoint.txid).expect("txid"),
        coin.outpoint.vout,
    )
    .expect("outpoint")
}

pub(crate) fn program(bundle: &FixtureBundle) -> Vec<u8> {
    bundle
        .pin()
        .output_script(bundle.target())
        .expect("program")
}

impl Planner {
    pub(crate) fn new(realize: Realize) -> Self {
        Self::staged(false, realize)
    }

    pub(crate) fn staged(include_unprobed: bool, realize: Realize) -> Self {
        let bundle = fixture_bundle().expect("candidate");
        let rows: Vec<_> = roster(&bundle).expect("roster").into_values().collect();
        let jobs = ControlFamily::ALL
            .into_iter()
            .map(|family| Job { family, row: None })
            .chain(rows.iter().filter(|row| row.recipe.is_ok() && (include_unprobed || matches!(candidate::compact_campaign::representability(row), candidate::compact_campaign::Representability::Representable { .. }))).map(|row| Job {
                family: row.control,
                row: Some(row.clone()),
            }))
            .collect();
        Self {
            realize,
            additional_realize: None,
            second: false,
            second_asset: None,
            extra_signer: None,
            signing_index: 0,
            rows,
            jobs,
            stage: Stage::Issue,
            index: 0,
            asset: None,
            bundle: None,
            program: program(&bundle),
            sponsors: Vec::new(),
            sponsor_index: 0,
            active_sponsor: None,
            control: Vec::new(),
            mutant: Vec::new(),
            pending: None,
            exchanges: Vec::new(),
            accepted_families: BTreeSet::new(),
            suspended: BTreeSet::new(),
            findings: Vec::new(),
            failure: None,
        }
    }

    pub(crate) fn second(realize: AdditionalRealize) -> Self {
        let mut planner = Self::new(Box::new(|_, _, _, _| Err("first schedule mutation".into())));
        planner.second = true;
        planner.additional_realize = Some(realize);
        planner.rows = candidate::compact_campaign_second::roster(&fixture_bundle().expect("candidate"))
            .expect("roster").into_values().collect();
        planner.jobs = ControlFamily::ALL.into_iter().map(|family| Job { family, row: None })
            .chain(planner.rows.iter().filter(|row| row.recipe.is_ok_and(candidate::compact_campaign_second::additional))
                .map(|row| Job { family: row.control, row: Some(row.clone()) })).collect();
        planner
    }

    fn sponsor_count(&self) -> usize {
        self.jobs.iter().map(|job| usize::from(job.family.sponsored)
            + usize::from(job.row.as_ref().is_some_and(|row| row.recipe == Ok(ByteRecipe::TwoSponsors)))).sum()
    }

    pub(crate) fn funding(
        &self,
        name: &str,
        issue: bool,
        outputs: u8,
        amount: u64,
    ) -> OperationStep {
        OperationStep::new(
            name,
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: issue,
                asset: if issue {
                    None
                } else {
                    self.asset.map(|asset| {
                        let mut display = asset;
                        display.reverse();
                        hex_bytes(&display)
                    })
                },
                output_program: self.program.clone(),
                outputs,
                amount_per_output: amount,
            })),
        )
    }

    fn next_job(&mut self) {
        self.index += 1;
        self.stage = Stage::Fund;
        self.active_sponsor = None;
        self.extra_signer = None;
        self.signing_index = 0;
        self.control.clear();
        self.mutant.clear();
    }

    #[allow(
        clippy::too_many_lines,
        reason = "construction keeps the control and malformed assembly adjacent"
    )]
    pub(crate) fn construct(&mut self, coins: &[FundedOutput]) -> Result<(), String> {
        let job = &self.jobs[self.index];
        let family = job.family;
        let bundle = self.bundle.as_ref().ok_or("bundle missing")?;
        let mut display_asset = bundle.closed_asset();
        display_asset.reverse();
        let display_asset = hex_bytes(&display_asset);
        let extra = usize::from(
            job.row
                .as_ref()
                .is_some_and(|row| row.recipe == Ok(ByteRecipe::FiveInputs)),
        );
        if coins.len() != usize::from(family.ash) + extra
            || coins.iter().any(|coin| {
                coin.amount_satoshis != 100
                    || coin.script != hex_bytes(&self.program)
                    || coin.asset != display_asset
            })
            || coins
                .iter()
                .map(|coin| (&coin.outpoint.txid, coin.outpoint.vout))
                .collect::<BTreeSet<_>>()
                .len()
                != coins.len()
        {
            return Err("funding fields disagree".into());
        }
        let sponsor = if !family.sponsored {
            SponsorCase::Absent
        } else if family.change {
            SponsorCase::sponsored_with_change(1)
        } else {
            SponsorCase::sponsored(1)
        };
        let case = semantic_case(
            SemanticFixtureId::new(
                "compact-campaign",
                u32::try_from(self.index).map_err(|e| e.to_string())?,
            ),
            candidate::matrix::POSITIVE_SEMANTIC[0],
            &vec![100; usize::from(family.ash)],
            sponsor,
        )
        .map_err(|e| format!("{e:?}"))?;
        let funding = AshFunding::new(
            vector_id(&case),
            coins[..usize::from(family.ash)]
                .iter()
                .map(outpoint)
                .collect(),
        )
        .map_err(|e| format!("{e:?}"))?;
        let vector = if family.sponsored {
            let coin = self
                .sponsors
                .get(self.sponsor_index)
                .ok_or("sponsor missing")?
                .clone();
            self.sponsor_index += 1;
            let tasks =
                sponsor_signing_requests(bundle, &case, &funding, std::slice::from_ref(&coin))
                    .map_err(|e| format!("{e:?}"))?;
            let signatures = tasks
                .iter()
                .map(|task| {
                    (
                        task.coin().outpoint(),
                        SponsorSignature::new(
                            task.request().transaction().to_vec(),
                            vec![Vec::new(), Vec::new()],
                        ),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let vector = materialize_sponsored(
                bundle,
                &case,
                &funding,
                std::slice::from_ref(&coin),
                &signatures,
            )
            .map_err(|e| format!("{e:?}"))?;
            self.active_sponsor = Some(coin);
            vector
        } else {
            materialize(bundle, &case, &funding).map_err(|e| format!("{e:?}"))?
        };
        self.control = vector.bytes().to_vec();
        if self.second && job.row.is_some() {
            self.stage = Stage::Extra;
            return Ok(());
        }
        if let Some(row) = &job.row {
            self.mutant = (self.realize)(
                &self.control,
                row.recipe.map_err(|e| format!("{e:?}"))?,
                coins.last().map(outpoint),
                family.ash,
            )?;
        }
        self.stage = if job.row.is_some() {
            Stage::SignMutant
        } else {
            Stage::SignControl
        };
        Ok(())
    }

    fn extra_funding(&self, name: &str, recipe: ByteRecipe) -> Result<OperationStep, String> {
        let tx = TargetTransaction::decode(&self.control).map_err(|e| format!("{e:?}"))?;
        let mut asset = if recipe == ByteRecipe::AshInputObject {
            self.asset.ok_or("asset missing")?
        } else { self.second_asset.ok_or("second asset missing")? };
        asset.reverse();
        let amount = match recipe {
            ByteRecipe::AshInputAsset | ByteRecipe::AshInputObject => 100,
            ByteRecipe::SponsorInputAsset => self.active_sponsor.as_ref().ok_or("sponsor missing")?.amount(),
            ByteRecipe::SuccessorAsset | ByteRecipe::ChangeAsset => {
                let index = usize::from(recipe == ByteRecipe::ChangeAsset);
                let transaction::ValueField::Explicit(value) = tx.outputs()[index].value() else { return Err("nonexplicit output".into()); };
                value
            }
            _ => return Err("unexpected extra recipe".into()),
        };
        let program = match recipe {
            ByteRecipe::AshInputAsset => self.program.clone(),
            ByteRecipe::SponsorInputAsset => self.active_sponsor.as_ref().ok_or("sponsor missing")?.program().to_vec(),
            _ => vec![0x51],
        };
        Ok(OperationStep::new(&format!("{name}/extra"), OperationSubject::Funding(Box::new(TargetFundingSubject {
            issue_asset: false, asset: Some(hex_bytes(&asset)), output_program: program, outputs: 1, amount_per_output: amount,
        }))))
    }

    fn realize_additional(&mut self, coin: &FundedOutput) -> Result<(), String> {
        let job = &self.jobs[self.index];
        let recipe = job.row.as_ref().ok_or("row missing")?.recipe.map_err(|e| format!("{e:?}"))?;
        if matches!(recipe, ByteRecipe::SponsorInputAsset | ByteRecipe::TwoSponsors) {
            let program = self.active_sponsor.as_ref().ok_or("sponsor missing")?.program().to_vec();
            if coin.script != hex_bytes(&program) { return Err("signer program differs".into()); }
            self.extra_signer = Some(SponsorCoin::new(outpoint(coin), coin.amount_satoshis, program));
        }
        self.mutant = self.additional_realize.as_mut().ok_or("additional realization missing")?(
            &self.control, recipe, coin, job.family.ash,
        )?;
        self.signing_index = 0;
        self.stage = Stage::SignMutant;
        Ok(())
    }

    #[allow(
        clippy::too_many_lines,
        reason = "one exhaustive state transition table"
    )]
    pub(crate) fn settle(
        &mut self,
        step: &OperationStep,
        response: &NativeOperationResponse,
    ) -> Result<(), String> {
        let accepted = response.observed_layer == ObservedOutcomeLayer::Accepted;
        match self.stage {
            Stage::Issue => {
                if !accepted {
                    return Err("issuance failed".into());
                }
                let mut asset = hex32(
                    response
                        .issued_asset
                        .as_deref()
                        .ok_or("issued asset absent")?,
                );
                asset.reverse();
                self.asset = Some(asset);
                self.stage = if self.second { Stage::IssueSecond } else { Stage::Sponsor };
            }
            Stage::IssueSecond => {
                if !accepted { return Err("second issuance failed".into()); }
                let mut asset = hex32(response.issued_asset.as_deref().ok_or("second asset absent")?);
                asset.reverse();
                if Some(asset) == self.asset { return Err("issued assets coincide".into()); }
                self.second_asset = Some(asset);
                self.stage = Stage::Sponsor;
            }
            Stage::Sponsor => {
                if !accepted {
                    return Err("sponsor funding failed".into());
                }
                let reserve = response.funded_outputs.first().ok_or("reserve missing")?;
                if response.funded_outputs.len()
                    != self.sponsor_count()
                    || response
                        .funded_outputs
                        .iter()
                        .any(|coin| coin.asset != reserve.asset || coin.amount_satoshis != 100_000)
                    || response
                        .funded_outputs
                        .iter()
                        .map(|coin| (&coin.outpoint.txid, coin.outpoint.vout))
                        .collect::<BTreeSet<_>>()
                        .len()
                        != response.funded_outputs.len()
                {
                    return Err("sponsor funding fields disagree".into());
                }
                let mut asset = hex32(&reserve.asset);
                asset.reverse();
                let bundle = ceremony_bundle(self.asset.ok_or("asset missing")?, asset)
                    .map_err(|e| format!("{e:?}"))?;
                self.program = program(&bundle);
                self.bundle = Some(bundle);
                for coin in &response.funded_outputs {
                    let bytes = coin
                        .script
                        .as_bytes()
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|pair| {
                            u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII"), 16)
                                .expect("hex")
                        })
                        .collect();
                    self.sponsors.push(SponsorCoin::new(
                        outpoint(coin),
                        coin.amount_satoshis,
                        bytes,
                    ));
                }
                self.stage = Stage::Derive;
            }
            Stage::Derive => {
                if !accepted
                    || response
                        .funded_outputs
                        .first()
                        .is_none_or(|coin| coin.script != hex_bytes(&self.program))
                {
                    return Err("constructor probe failed".into());
                }
                self.stage = Stage::Fund;
            }
            Stage::Fund => {
                if !accepted {
                    return Err("ASH funding failed".into());
                }
                self.construct(&response.funded_outputs)?;
            }
            Stage::Extra => {
                let OperationSubject::Funding(subject) = step.subject() else { return Err("extra funding subject".into()); };
                let coin = response.funded_outputs.first().ok_or("extra coin absent")?;
                if !accepted || response.funded_outputs.len() != 1
                    || Some(&coin.asset) != subject.asset.as_ref()
                    || coin.amount_satoshis != subject.amount_per_output
                    || coin.script != hex_bytes(&subject.output_program) {
                    return Err("extra funding fields disagree".into());
                }
                self.realize_additional(coin)?;
            }
            Stage::SignMutant | Stage::SignControl => {
                let OperationSubject::SponsorSigning(subject) = step.subject() else {
                    return Err("signing subject missing".into());
                };
                if !accepted
                    || response.signature_bound_to.as_deref()
                        != Some(subject.finalized_transaction.as_slice())
                    || response.sponsor_witness.len() != 2
                {
                    return Err("signing binding failed".into());
                }
                let bytes = if matches!(self.stage, Stage::SignMutant) {
                    &mut self.mutant
                } else {
                    &mut self.control
                };
                *bytes = attach(
                    bytes,
                    subject.sponsor_input_index,
                    &response.sponsor_witness,
                )?;
                self.signing_index += 1;
                if !self.second {
                    self.stage = if matches!(self.stage, Stage::SignMutant) {
                        Stage::SubmitMutant
                    } else { Stage::SubmitControl };
                }
            }
            Stage::SubmitMutant => {
                let row = self.jobs[self.index].row.as_ref().ok_or("row missing")?;
                self.findings.push(format!(
                    "mutant {} {:?} {}",
                    requirement_name(&row.manifest.requirement),
                    response.observed_layer,
                    candidate::compact_campaign::matches(row, response.observed_layer)
                ));
                if accepted || response.observed_layer == ObservedOutcomeLayer::RelayPolicyRejection
                {
                    self.suspended.insert(row.control);
                    self.next_job();
                } else {
                    self.signing_index = 0;
                    self.stage = Stage::SignControl;
                }
            }
            Stage::SubmitControl => {
                let family = self.jobs[self.index].family;
                if accepted {
                    verify_readback(&self.control, response)?;
                    self.accepted_families.insert(family);
                } else {
                    self.suspended.insert(family);
                }
                self.findings.push(format!(
                    "control {} {} {:?}",
                    self.index,
                    family.name(),
                    response.observed_layer
                ));
                self.next_job();
            }
            Stage::Done => return Err("response after completion".into()),
        }
        Ok(())
    }

    #[allow(
        clippy::too_many_lines,
        reason = "one exhaustive step table across both schedules"
    )]
    pub(crate) fn step(&mut self) -> Result<Option<OperationStep>, String> {
        loop {
            let name = format!("job-{}", self.index);
            return Ok(Some(match self.stage {
                Stage::Issue => self.funding("issue", true, 1, 1),
                Stage::IssueSecond => self.funding("issue-second", true, 1, 1),
                Stage::Sponsor => OperationStep::new(
                    "sponsor",
                    OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                        sponsor_outputs: u8::try_from(
                            self.sponsor_count(),
                        )
                        .map_err(|e| e.to_string())?,
                        amount_per_sponsor_output: 100_000,
                    })),
                ),
                Stage::Derive => self.funding("constructor", false, 1, 1),
                Stage::Fund => {
                    let Some(job) = self.jobs.get(self.index) else {
                        self.stage = Stage::Done;
                        return Ok(None);
                    };
                    if job.row.is_some() && self.suspended.contains(&job.family) {
                        self.next_job();
                        continue;
                    }
                    let extra = u8::from(
                        job.row
                            .as_ref()
                            .is_some_and(|row| row.recipe == Ok(ByteRecipe::FiveInputs)),
                    );
                    self.funding(&format!("{name}/fund"), false, job.family.ash + extra, 100)
                }
                Stage::Extra => {
                    let recipe = self.jobs[self.index].row.as_ref().ok_or("row missing")?.recipe.map_err(|e| format!("{e:?}"))?;
                    if recipe == ByteRecipe::TwoSponsors {
                        let coin = self.sponsors.get(self.sponsor_index).ok_or("second sponsor absent")?.clone();
                        self.sponsor_index += 1;
                        let mut asset = self.bundle.as_ref().ok_or("bundle absent")?.reserve_asset();
                        asset.reverse();
                        let funded = FundedOutput {
                            outpoint: WireOutpoint { txid: coin.outpoint().txid().to_target_display(), vout: coin.outpoint().index() },
                            asset: hex_bytes(&asset), amount_satoshis: coin.amount(), script: hex_bytes(coin.program()),
                        };
                        self.realize_additional(&funded)?;
                        continue;
                    }
                    self.extra_funding(&name, recipe)?
                }
                Stage::SignMutant | Stage::SignControl => {
                    let mutant = matches!(self.stage, Stage::SignMutant);
                    let omit = mutant
                        && self.jobs[self.index]
                            .row
                            .as_ref()
                            .is_some_and(|row| row.recipe == Ok(ByteRecipe::MissingAuthorization));
                    let signer = if self.second && self.signing_index > 0 {
                        if mutant && self.signing_index == 1 { self.extra_signer.as_ref() } else { None }
                    } else { self.active_sponsor.as_ref() };
                    let Some(coin) = signer.filter(|_| !omit) else {
                        self.stage = if mutant {
                            Stage::SubmitMutant
                        } else {
                            Stage::SubmitControl
                        };
                        continue;
                    };
                    let bytes = if mutant { &self.mutant } else { &self.control };
                    let transaction =
                        TargetTransaction::decode(bytes).map_err(|e| format!("{e:?}"))?;
                    let index = transaction
                        .inputs()
                        .iter()
                        .position(|input| input.outpoint() == coin.outpoint())
                        .ok_or("sponsor input missing")?;
                    let signing_name = if self.second {
                        format!("{name}/sign-{}/{}", if mutant { "mutant" } else { "control" }, self.signing_index)
                    } else { format!("{name}/sign-{}", if mutant { "mutant" } else { "control" }) };
                    OperationStep::new(
                        &signing_name,
                        OperationSubject::SponsorSigning(Box::new(TargetSponsorSigningSubject {
                            finalized_transaction: bytes.clone(),
                            sponsor_input_index: u16::try_from(index).map_err(|e| e.to_string())?,
                            sponsor_outpoint: WireOutpoint {
                                txid: coin.outpoint().txid().to_target_display(),
                                vout: coin.outpoint().index(),
                            },
                            sighash_profile: WireSighashProfile::AllInputsAllOutputs,
                        })),
                    )
                }
                Stage::SubmitMutant | Stage::SubmitControl => {
                    let mutant = matches!(self.stage, Stage::SubmitMutant);
                    OperationStep::new(
                        &format!(
                            "{name}/submit-{}",
                            if mutant { "mutant" } else { "control" }
                        ),
                        OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                            transaction_bytes: if mutant {
                                self.mutant.clone()
                            } else {
                                self.control.clone()
                            },
                        })),
                    )
                }
                Stage::Done => return Ok(None),
            }));
        }
    }
}

impl TargetOperationPlanner for Planner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        let result = (|| {
            if let Some((case, response)) = previous {
                let step = self.pending.take().ok_or("unexpected response")?;
                if step.case() != case || &response.case != case {
                    return Err("response identity mismatch".into());
                }
                self.exchanges.push((step.clone(), response.clone()));
                self.settle(&step, response)?;
            } else if self.pending.is_some() {
                return Err("response missing".into());
            }
            let step = self.step()?;
            self.pending.clone_from(&step);
            Ok(step)
        })();
        result.map_err(|error| {
            self.failure = Some(error);
            PlanRefused
        })
    }
}

pub(crate) fn rebuild(
    original: &TargetTransaction,
    inputs: Vec<TargetInput>,
    outputs: Vec<TargetOutput>,
    witnesses: Vec<InputWitness>,
) -> Result<Vec<u8>, String> {
    TargetTransaction::new(
        original.version(),
        inputs,
        outputs,
        original.lock_time(),
        witnesses,
    )
    .map(|tx| tx.encode())
    .map_err(|e| format!("{e:?}"))
}

pub(crate) fn attach(bytes: &[u8], index: u16, stack: &[Vec<u8>]) -> Result<Vec<u8>, String> {
    let tx = TargetTransaction::decode(bytes).map_err(|e| format!("{e:?}"))?;
    let mut witnesses = tx.witnesses().to_vec();
    *witnesses
        .get_mut(usize::from(index))
        .ok_or("signing index")? = InputWitness::new(stack.to_vec());
    rebuild(&tx, tx.inputs().to_vec(), tx.outputs().to_vec(), witnesses)
}

pub(crate) fn verify_readback(
    bytes: &[u8],
    response: &NativeOperationResponse,
) -> Result<(), String> {
    let readback = response
        .mined_readback
        .as_ref()
        .ok_or("mined readback absent")?;
    let tx = TargetTransaction::decode(bytes).map_err(|e| format!("{e:?}"))?;
    let identity = Txid::from_internal(target_elements_conformance::constructor::tagged::sha256(
        &target_elements_conformance::constructor::tagged::sha256(&tx.encode_without_witness()),
    ))
    .to_target_display();
    let witness = Txid::from_internal(target_elements_conformance::constructor::tagged::sha256(
        &target_elements_conformance::constructor::tagged::sha256(bytes),
    ))
    .to_target_display();
    if readback.raw_transaction != bytes
        || readback.transaction_id != identity
        || readback.witness_transaction_id != witness
        || response.accepted_txid.as_deref() != Some(identity.as_str())
    {
        return Err("readback differs".into());
    }
    Ok(())
}
