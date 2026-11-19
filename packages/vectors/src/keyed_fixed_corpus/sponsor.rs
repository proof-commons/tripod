//! S8c joins over a separately captured sponsor and old-leaf copy packet.
//! The fixture loader and retained source resolver remain in the parent owner.

use super::{CorpusRefusal, Packet};
use elements::bitcoin::hashes::Hash as _;
use elements::hashes::sha256d;
use elements::secp256k1_zkp::{
    Generator, PedersenCommitment, Secp256k1, Tag, ZERO_TWEAK, verify_commitments_sum_to_equal,
};
use serde::Deserialize;
use target_elements::reviewed_elements_tapscript;
use target_elements_conformance::confidential_funding::RangeproofVerifier;
use target_elements_conformance::confidential_oracles::ReferenceRangeproofVerifier;
use target_elements_conformance::constructor::{FixtureTapTree, construct};
use target_elements_conformance::protocol::{
    NATIVE_PROTOCOL_SCHEMA, NativeOperationRequest, NativeOperationResponse, ObservedOutcomeLayer,
    OperationStepKind, OperationSubject,
};
use target_elements_conformance::sponsor_parent::bind_sponsor_parent;
use transaction::{AssetField, TargetOutput, TargetTransaction, Txid, ValueField};
use vectors::keyed_fixed_capture::{CaptureFamily, CaptureSubject, subjects};
use vectors::keyed_fixed_manifest::ValidatedFreeze;

fn txid(bytes: &[u8]) -> Result<Txid, CorpusRefusal> {
    let transaction = TargetTransaction::decode(bytes).map_err(|_| CorpusRefusal::Record)?;
    Ok(Txid::from_internal(
        sha256d::Hash::hash(&transaction.encode_without_witness()).to_byte_array(),
    ))
}

fn first_edition_source() -> Result<transaction::keyed_taproot::KeyedCommitment, CorpusRefusal> {
    use architecture::{
        keyed::KeyedManifest,
        semantic::{KeyedOperationId, SemanticOperationId},
    };
    use compiler::{
        AnalysisPolicy, ProofSearchLimits, bind_input,
        keyed_plan::{PlacementSearchLimits, plan},
    };
    use linker::{keyed_bundle::link_keyed_bundle, keyed_reference::keyed_parameter_offers};
    use tapscript::{
        keyed_bundle::{KeyedBundleSelection, emit_keyed_bundle},
        keyed_constructor::KeyedLeafRecipe,
    };
    let target = reviewed_elements_tapscript().map_err(|_| CorpusRefusal::Witness)?;
    let manifest = KeyedManifest::contract().map_err(|_| CorpusRefusal::Witness)?;
    let declaration = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .map_err(|_| CorpusRefusal::Witness)?
        .validate(&manifest)
        .map_err(|_| CorpusRefusal::Witness)?;
    let limit = std::num::NonZeroU64::new(100_000).ok_or(CorpusRefusal::Witness)?;
    let input = bind_input(
        &manifest,
        declaration,
        vec![SemanticOperationId::Keyed(KeyedOperationId::Source)],
        AnalysisPolicy::strict(ProofSearchLimits::new(limit, limit)),
    )
    .map_err(|_| CorpusRefusal::Witness)?;
    let planned = plan(&input, PlacementSearchLimits::new(limit, limit))
        .map_err(|_| CorpusRefusal::Witness)?;
    let projection = planned.project();
    let selection = KeyedBundleSelection {
        binding: projection.binding.clone(),
        scope: projection.scope.clone(),
        proof: 0,
        placements: projection.plans[0]
            .operations
            .keys()
            .map(|id| (*id, 0))
            .collect(),
    };
    let recipe = KeyedLeafRecipe::new([42; 32]);
    let emission = emit_keyed_bundle(&target, &planned, selection, recipe)
        .map_err(|_| CorpusRefusal::Witness)?;
    let linked = link_keyed_bundle(&target, &emission, &keyed_parameter_offers(&target, recipe))
        .map_err(|_| CorpusRefusal::Witness)?;
    transaction::keyed_taproot::commit_keyed_bundle(&target, &linked)
        .map_err(|_| CorpusRefusal::Witness)
}

#[derive(Deserialize)]
struct LegacyRecord {
    subject: CaptureSubject,
    freeze: [u8; 32],
    submitted: Vec<u8>,
    parents: Vec<Vec<u8>>,
    response: NativeOperationResponse,
}

fn commitment(
    output: &TargetOutput,
    asset: &[u8; 32],
) -> Result<PedersenCommitment, CorpusRefusal> {
    if !matches!(output.asset(), AssetField::Explicit(id) if id.internal() == asset) {
        return Err(CorpusRefusal::Witness);
    }
    let secp = Secp256k1::new();
    let generator = Generator::new_unblinded(&secp, Tag::from(*asset));
    match output.value() {
        ValueField::Explicit(value) => {
            Ok(PedersenCommitment::new(&secp, value, ZERO_TWEAK, generator))
        }
        ValueField::Commitment(bytes) => {
            PedersenCommitment::from_slice(&bytes).map_err(|_| CorpusRefusal::Witness)
        }
        _ => Err(CorpusRefusal::Witness),
    }
}

fn proof(
    output: &TargetOutput,
    witness: &transaction::bytes::OutputWitness,
    asset: &[u8; 32],
) -> Result<(), CorpusRefusal> {
    if !witness.surjection_proof().is_empty() {
        return Err(CorpusRefusal::Witness);
    }
    match output.value() {
        ValueField::Explicit(_) if witness.range_proof().is_empty() => Ok(()),
        ValueField::Commitment(bytes) if !witness.range_proof().is_empty() => {
            let printed_asset = vectors::maturity_observer::encode_hex(
                &asset.iter().rev().copied().collect::<Vec<_>>(),
            );
            if ReferenceRangeproofVerifier::new().verifies(
                witness.range_proof(),
                &bytes,
                &printed_asset,
                output.program(),
            ) {
                Ok(())
            } else {
                Err(CorpusRefusal::Witness)
            }
        }
        _ => Err(CorpusRefusal::Witness),
    }
}

fn proof_and_tally(
    submitted: &[u8],
    parents: &[Vec<u8>],
    asset: &[u8; 32],
) -> Result<(), CorpusRefusal> {
    let transaction = TargetTransaction::decode(submitted).map_err(|_| CorpusRefusal::Record)?;
    if transaction.inputs().len() != parents.len()
        || transaction.outputs().len() != transaction.output_witnesses().len()
    {
        return Err(CorpusRefusal::Record);
    }
    let mut inputs = Vec::new();
    for (input, bytes) in transaction.inputs().iter().zip(parents) {
        let parent = TargetTransaction::decode(bytes).map_err(|_| CorpusRefusal::Record)?;
        if input.outpoint().txid() != txid(bytes)? {
            return Err(CorpusRefusal::Record);
        }
        let index = input.outpoint().index() as usize;
        let output = parent.outputs().get(index).ok_or(CorpusRefusal::Record)?;
        let witness = parent
            .output_witnesses()
            .get(index)
            .ok_or(CorpusRefusal::Record)?;
        proof(output, witness, asset)?;
        inputs.push(commitment(output, asset)?);
    }
    let mut outputs = Vec::new();
    for (output, witness) in transaction
        .outputs()
        .iter()
        .zip(transaction.output_witnesses())
    {
        proof(output, witness, asset)?;
        outputs.push(commitment(output, asset)?);
    }
    if !verify_commitments_sum_to_equal(&Secp256k1::new(), &inputs, &outputs) {
        return Err(CorpusRefusal::Witness);
    }
    Ok(())
}

/// Join the one admitted sponsor payment separately from fixed-form members.
/// # Errors
/// Refuses an unbound or unsupported payment, proof, or value tally.
pub(super) fn check_sponsor_record(
    record: &super::Record,
    freeze: &ValidatedFreeze,
) -> Result<(), CorpusRefusal> {
    if record.subject == CaptureSubject::SponsorPayment {
        if record.row.response.observed_layer != ObservedOutcomeLayer::Accepted
            || record.row.parent_bytes.len() != 1
        {
            return Err(CorpusRefusal::Record);
        }
        let transaction =
            TargetTransaction::decode(&record.row.submitted).map_err(|_| CorpusRefusal::Record)?;
        if transaction.outputs().first().is_none_or(|output| {
            output.program() != freeze.manifest().descriptor.output_program
                || !matches!(output.value(), ValueField::Commitment(_))
        }) {
            return Err(CorpusRefusal::Record);
        }
        check_sponsor_signature(record)?;
    }
    check_sponsor_digest(record)?;
    proof_and_tally(
        &record.row.submitted,
        &record.row.parent_bytes,
        &record.asset,
    )?;
    Ok(())
}

fn check_sponsor_digest(record: &super::Record) -> Result<(), CorpusRefusal> {
    let transaction =
        TargetTransaction::decode(&record.row.submitted).map_err(|_| CorpusRefusal::Record)?;
    let input = transaction
        .inputs()
        .get(record.sponsor_index)
        .ok_or(CorpusRefusal::Record)?;
    let parent = TargetTransaction::decode(
        record
            .row
            .parent_bytes
            .get(record.sponsor_index)
            .ok_or(CorpusRefusal::Record)?,
    )
    .map_err(|_| CorpusRefusal::Record)?;
    let spent = parent
        .outputs()
        .get(input.outpoint().index() as usize)
        .ok_or(CorpusRefusal::Record)?;
    let ValueField::Explicit(value) = spent.value() else {
        return Err(CorpusRefusal::Witness);
    };
    let digest = transaction::keyed_fixed_sponsor::keyed_fixed_sponsor_digest(
        &transaction.encode_without_witness(),
        record.sponsor_index,
        spent,
    )
    .map_err(|_| CorpusRefusal::Witness)?;
    let reference: elements::Transaction =
        elements::encode::deserialize(&record.row.submitted).map_err(|_| CorpusRefusal::Record)?;
    let script = elements::Script::new_p2pkh(
        &elements::PubkeyHash::from_slice(&spent.program()[2..])
            .map_err(|_| CorpusRefusal::Witness)?,
    );
    let independent = elements::sighash::SighashCache::new(&reference)
        .segwitv0_sighash(
            record.sponsor_index,
            &script,
            elements::confidential::Value::Explicit(value),
            elements::EcdsaSighashType::All,
        )
        .to_byte_array();
    if digest != independent {
        return Err(CorpusRefusal::Witness);
    }
    Ok(())
}

fn check_sponsor_signature(record: &super::Record) -> Result<(), CorpusRefusal> {
    let transaction =
        TargetTransaction::decode(&record.row.submitted).map_err(|_| CorpusRefusal::Record)?;
    let input = transaction
        .inputs()
        .get(record.sponsor_index)
        .ok_or(CorpusRefusal::Record)?;
    let parent = TargetTransaction::decode(
        record
            .row
            .parent_bytes
            .get(record.sponsor_index)
            .ok_or(CorpusRefusal::Record)?,
    )
    .map_err(|_| CorpusRefusal::Record)?;
    let spent = parent
        .outputs()
        .get(input.outpoint().index() as usize)
        .ok_or(CorpusRefusal::Record)?;
    let witness = transaction
        .witnesses()
        .get(record.sponsor_index)
        .ok_or(CorpusRefusal::Record)?;
    transaction::keyed_fixed_sponsor::verify_keyed_fixed_sponsor(
        &transaction.encode_without_witness(),
        record.sponsor_index,
        spent,
        witness.stack(),
    )
    .map_err(|_| CorpusRefusal::Witness)
}

impl Packet {
    /// Re-admit the selected S8c family after checked acquisition. A sponsor
    /// payment is evidence of funding and proof, never a fixed-form member.
    /// # Errors
    /// Returns the first exact transport, sponsor, proof, copy or identity refusal.
    pub fn admit_s8c(&self) -> Result<usize, CorpusRefusal> {
        match self.family {
            CaptureFamily::Sponsor => self.admit_all(),
            CaptureFamily::LegacyCopy => self.admit_legacy_copy(),
            _ => Err(CorpusRefusal::Census),
        }
    }

    fn legacy_record(&self, ordinal: usize) -> Result<LegacyRecord, CorpusRefusal> {
        let name = format!("legacy-{ordinal:05}");
        let bytes = self.family_file(&format!("{name}.json"))?;
        let record: LegacyRecord =
            serde_json::from_slice(bytes).map_err(|_| CorpusRefusal::Record)?;
        if self.family_file(&format!("{name}.transaction"))? != record.submitted {
            return Err(CorpusRefusal::Record);
        }
        for (index, parent) in record.parents.iter().enumerate() {
            if self.family_file(&format!("{name}-parent-{index}.transaction"))? != parent {
                return Err(CorpusRefusal::Record);
            }
        }
        Ok(record)
    }

    fn check_legacy_transport(
        &self,
        ordinal: usize,
        record: &LegacyRecord,
    ) -> Result<(), CorpusRefusal> {
        let fund = ordinal - 2;
        let sign = ordinal - 1;
        let fund_request: NativeOperationRequest =
            serde_json::from_slice(self.family_file(&format!("exchange-{fund:05}-request.json"))?)
                .map_err(|_| CorpusRefusal::Record)?;
        let fund_response: NativeOperationResponse =
            serde_json::from_slice(self.family_file(&format!("exchange-{fund:05}-response.json"))?)
                .map_err(|_| CorpusRefusal::Record)?;
        let sign_request: NativeOperationRequest =
            serde_json::from_slice(self.family_file(&format!("exchange-{sign:05}-request.json"))?)
                .map_err(|_| CorpusRefusal::Record)?;
        let sign_response: NativeOperationResponse =
            serde_json::from_slice(self.family_file(&format!("exchange-{sign:05}-response.json"))?)
                .map_err(|_| CorpusRefusal::Record)?;
        let submit_request: NativeOperationRequest = serde_json::from_slice(
            self.family_file(&format!("exchange-{ordinal:05}-request.json"))?,
        )
        .map_err(|_| CorpusRefusal::Record)?;
        let submit_response: NativeOperationResponse = serde_json::from_slice(
            self.family_file(&format!("exchange-{ordinal:05}-response.json"))?,
        )
        .map_err(|_| CorpusRefusal::Record)?;
        if fund_request.case != fund_response.case
            || sign_request.case != sign_response.case
            || submit_request.case != submit_response.case
            || [
                fund_request.schema,
                sign_request.schema,
                submit_request.schema,
            ] != [NATIVE_PROTOCOL_SCHEMA; 3]
            || [
                fund_response.schema,
                sign_response.schema,
                submit_response.schema,
            ] != [NATIVE_PROTOCOL_SCHEMA; 3]
            || fund_request.case.operation != OperationStepKind::FundSponsor
            || sign_request.case.operation != OperationStepKind::SignSponsor
            || submit_request.case.operation != OperationStepKind::Submit
            || submit_response != record.response
            || sign_response.signature_bound_to.is_none()
        {
            return Err(CorpusRefusal::Record);
        }
        for response in [&fund_response, &sign_response, &submit_response] {
            response
                .validate_shape()
                .map_err(|_| CorpusRefusal::Record)?;
        }
        let OperationSubject::Submission(submission) = &submit_request.subject else {
            return Err(CorpusRefusal::Record);
        };
        if submission.transaction_bytes != record.submitted {
            return Err(CorpusRefusal::Record);
        }
        Self::check_legacy_payment(record, &fund_response, &sign_request, &sign_response)?;
        Ok(())
    }

    fn check_legacy_payment(
        record: &LegacyRecord,
        fund_response: &NativeOperationResponse,
        sign_request: &NativeOperationRequest,
        sign_response: &NativeOperationResponse,
    ) -> Result<(), CorpusRefusal> {
        Self::check_response_outcome(&record.submitted, &record.response)?;
        if record.response.observed_layer != ObservedOutcomeLayer::Accepted
            || record
                .response
                .mined_readback
                .as_ref()
                .is_none_or(|readback| {
                    readback.raw_transaction != record.submitted
                        || record.response.accepted_txid.as_deref()
                            != Some(readback.transaction_id.as_str())
                })
        {
            return Err(CorpusRefusal::Record);
        }
        let transaction =
            TargetTransaction::decode(&record.submitted).map_err(|_| CorpusRefusal::Record)?;
        let sponsor_index = record
            .parents
            .len()
            .checked_sub(1)
            .ok_or(CorpusRefusal::Record)?;
        let sponsor = transaction
            .inputs()
            .get(sponsor_index)
            .ok_or(CorpusRefusal::Record)?;
        let parent = TargetTransaction::decode(&record.parents[sponsor_index])
            .map_err(|_| CorpusRefusal::Record)?;
        let spent = parent
            .outputs()
            .get(sponsor.outpoint().index() as usize)
            .ok_or(CorpusRefusal::Record)?;
        let OperationSubject::SponsorSigning(signing) = &sign_request.subject else {
            return Err(CorpusRefusal::Record);
        };
        let funded = fund_response
            .funded_outputs
            .iter()
            .find(|output| {
                output.outpoint.txid == sponsor.outpoint().txid().to_target_display()
                    && output.outpoint.vout == sponsor.outpoint().index()
            })
            .ok_or(CorpusRefusal::Record)?;
        let funding_readback = fund_response
            .mined_readback
            .as_ref()
            .ok_or(CorpusRefusal::Record)?;
        if sign_response.signature_bound_to.as_deref()
            != Some(transaction.encode_without_witness().as_slice())
            || sign_response.sponsor_witness != transaction.witnesses()[sponsor_index].stack()
            || signing.finalized_transaction != transaction.encode_without_witness()
            || usize::from(signing.sponsor_input_index) != sponsor_index
            || signing.sponsor_outpoint != funded.outpoint
            || bind_sponsor_parent(funding_readback, funded).map_err(|_| CorpusRefusal::Record)?
                != record.parents[sponsor_index]
        {
            return Err(CorpusRefusal::Record);
        }
        transaction::keyed_fixed_sponsor::verify_keyed_fixed_sponsor(
            &transaction.encode_without_witness(),
            sponsor_index,
            spent,
            transaction.witnesses()[sponsor_index].stack(),
        )
        .map_err(|_| CorpusRefusal::Witness)?;
        Ok(())
    }

    fn admit_legacy_copy(&self) -> Result<usize, CorpusRefusal> {
        let expected = vectors::keyed_final_native::capture_freeze_digest([42; 32])
            .map_err(|_| CorpusRefusal::Provenance)?;
        self.provenance(expected)?;
        let selected = subjects(CaptureFamily::LegacyCopy);
        let records = [3, 6, 9]
            .map(|ordinal| self.legacy_record(ordinal))
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        for (index, record) in records.iter().enumerate() {
            if record.subject != selected[index] || record.freeze != expected {
                return Err(CorpusRefusal::Record);
            }
            self.check_legacy_transport(3 * (index + 1), record)?;
            let transaction =
                TargetTransaction::decode(&record.submitted).map_err(|_| CorpusRefusal::Record)?;
            let AssetField::Explicit(asset) = transaction.outputs()[0].asset() else {
                return Err(CorpusRefusal::Witness);
            };
            proof_and_tally(&record.submitted, &record.parents, asset.internal())?;
        }
        Self::check_old_leaf(&records[0], &records[1])?;
        self.check_public_copy(&records[0], &records[2])?;
        self.check_funded_coin_census()?;
        tracing::info!(target: "vectors::keyed_fixed_corpus", family = "legacy-copy", records = 3, outcome = "completed", "s8c legacy copy rows admitted");
        Ok(3)
    }

    fn check_old_leaf(
        source: &LegacyRecord,
        continuation: &LegacyRecord,
    ) -> Result<(), CorpusRefusal> {
        let parent =
            TargetTransaction::decode(&source.submitted).map_err(|_| CorpusRefusal::Record)?;
        let parent_txid = txid(&source.submitted)?;
        let successor = TargetTransaction::decode(&continuation.submitted)
            .map_err(|_| CorpusRefusal::Record)?;
        if continuation.parents.first() != Some(&source.submitted)
            || successor.inputs().first().is_none_or(|input| {
                input.outpoint().txid() != parent_txid || input.outpoint().index() != 0
            })
        {
            return Err(CorpusRefusal::Record);
        }
        let witness = successor
            .witnesses()
            .first()
            .ok_or(CorpusRefusal::Witness)?
            .stack();
        let control = witness.last().ok_or(CorpusRefusal::Witness)?;
        let script = witness
            .get(witness.len().checked_sub(2).ok_or(CorpusRefusal::Witness)?)
            .ok_or(CorpusRefusal::Witness)?;
        let internal: [u8; 32] = control
            .get(1..33)
            .ok_or(CorpusRefusal::Witness)?
            .try_into()
            .map_err(|_| CorpusRefusal::Witness)?;
        let leaf = FixtureTapTree::leaf_of_version(control[0] & 0xfe, script.clone());
        let reconstructed =
            construct(&internal, &leaf, &leaf).map_err(|_| CorpusRefusal::Witness)?;
        let reviewed = first_edition_source()?;
        if reconstructed.output_program()
            != parent
                .outputs()
                .first()
                .ok_or(CorpusRefusal::Record)?
                .program()
            || reconstructed.control_block() != control
            || reviewed.output_program() != parent.outputs()[0].program()
            || reviewed.leaf() != script
            || reviewed.control().as_slice() != control.as_slice()
        {
            return Err(CorpusRefusal::Witness);
        }
        Ok(())
    }

    fn check_public_copy(
        &self,
        source: &LegacyRecord,
        copy: &LegacyRecord,
    ) -> Result<(), CorpusRefusal> {
        let input = self.family_file("legacy-public-input.transaction")?;
        let output = self.family_file("legacy-public-copy.transaction")?;
        let process = self.family_json("legacy-copy-process.json")?;
        let command = serde_json::json!([
            "/usr/bin/timeout",
            "--kill-after=5",
            "60",
            "/usr/bin/cp",
            "/capture/legacy-public-input.transaction",
            "/capture/legacy-public-copy.transaction"
        ]);
        if input != source.submitted
            || output != input
            || process["command"] != command
            || process["exit"] != 0
            || !self.family_file("legacy-copy.stdout")?.is_empty()
            || !self.family_file("legacy-copy.stderr")?.is_empty()
            || process["wall_micros"]
                .as_u64()
                .is_none_or(|wall| wall > 60_000_000)
        {
            return Err(CorpusRefusal::Record);
        }
        let original = TargetTransaction::decode(input).map_err(|_| CorpusRefusal::Record)?;
        let copied =
            TargetTransaction::decode(&copy.submitted).map_err(|_| CorpusRefusal::Record)?;
        if original.outputs() != copied.outputs()
            || original.output_witnesses() != copied.output_witnesses()
            || copy.parents.len() != 1
            || copied.inputs().len() != 1
        {
            return Err(CorpusRefusal::Witness);
        }
        Ok(())
    }
}
