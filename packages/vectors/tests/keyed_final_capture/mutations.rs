//! Untrusted subjects below sealed construction, always paired with its bytes.
use super::*;

impl<I: Exchange> Session<I> {
    fn raw_submission(&mut self, key: [u8; 32], schema: CaptureSchema, bytes: Vec<u8>) -> Vec<u8> {
        self.io.before_submission(CaptureRow {
            schema,
            key,
            freeze: capture_freeze_digest(key).unwrap(),
            digest: sha256(&bytes),
            submitted: bytes.clone(),
            boundary: schema.boundary(),
            expected: schema.outcome(),
            sealed_control: false,
        });
        let step = OperationStep::new(
            &format!(
                "untrusted-{schema:?}-{}",
                crate::common::hex_bytes(&sha256(&bytes))
            ),
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: bytes.clone(),
            })),
        );
        let response = self.io.exchange(step);
        assert_eq!(
            response.observed_layer,
            schema.outcome(),
            "{schema:?}: {:?}",
            response.observed_detail
        );
        if schema.outcome() == ObservedOutcomeLayer::Accepted {
            assert_eq!(response.mined_readback.unwrap().raw_transaction, bytes);
        }
        bytes
    }

    // A negative that changes non-witness bytes needs a new ordinary signature.
    // It cannot masquerade as a FinalSubject; only its accepted twin has that type.
    fn resign(&mut self, control: &FinalSubject, tx: &mut elements::Transaction) {
        let OperationSubject::SponsorSigning(mut request) =
            control.frozen().signing_step().subject().clone()
        else {
            panic!("signer")
        };
        let index = usize::from(request.sponsor_input_index);
        let complete = elements::encode::serialize(tx);
        request.finalized_transaction = TargetTransaction::decode(&complete)
            .unwrap()
            .encode_without_witness();
        let frozen = request.finalized_transaction.clone();
        let step = OperationStep::new(
            &format!("mutant-sign-{}", crate::common::hex_bytes(&sha256(&frozen))),
            OperationSubject::SponsorSigning(request),
        );
        let witness = self.sign(step);
        transaction::keyed_sponsor::verify_keyed_sponsor(
            &frozen,
            index,
            control.frozen().inputs()[index].parent().output(),
            &witness,
        )
        .unwrap();
        tx.input[index].witness.script_witness = witness.into();
    }

    pub(super) fn later_parent(&mut self, control: &FinalSubject) -> Vec<u8> {
        let mut tx: elements::Transaction = elements::encode::deserialize(control.bytes()).unwrap();
        let elements::confidential::Value::Explicit(change) = tx.output[1].value else {
            panic!("explicit")
        };
        tx.output[1].value = elements::confidential::Value::Explicit(change - 1000);
        tx.output.insert(1, tx.output[0].clone());
        self.resign(control, &mut tx);
        self.raw_submission(
            *control.frozen().key(),
            CaptureSchema::LaterParent,
            elements::encode::serialize(&tx),
        )
    }

    // The existing executor classifies key-path rejection only when EVERY input
    // is key-path. A sponsored transaction cannot meet that observation rule.
    // This raw pair is therefore explicit, unadmitted and has a single covenant
    // input; it does not change the sealed control or the executor classifier.
    pub(super) fn key_path_pair(&mut self, control: &FinalSubject) -> Vec<u8> {
        let mut tx: elements::Transaction = elements::encode::deserialize(control.bytes()).unwrap();
        assert!(control.frozen().inputs()[0].member());
        tx.input.truncate(1);
        let fee = tx.output.pop().unwrap();
        tx.output.truncate(1);
        tx.output[0].value = elements::confidential::Value::Explicit(546);
        tx.output.push(fee);
        tx.output[1].value = elements::confidential::Value::Explicit(454);
        let mut mutant = tx.clone();
        mutant.input[0].witness.script_witness = vec![vec![0; 64]].into();
        self.raw_submission(
            *control.frozen().key(),
            CaptureSchema::KeyPathAttempt,
            elements::encode::serialize(&mutant),
        );
        self.raw_submission(
            *control.frozen().key(),
            CaptureSchema::KeyPathControl,
            elements::encode::serialize(&tx),
        )
    }

    #[allow(
        clippy::too_many_lines,
        reason = "Explicit one-coordinate subject recipes remain together for review."
    )]
    pub(super) fn mutants(&mut self, control: &FinalSubject, schema: CaptureSchema) {
        use CaptureSchema as S;
        let subjects: &[CaptureSchema] = match (self.target, schema) {
            (CaptureTarget::Birth, S::LaterControl) => &[S::LaterBorn, S::Flagged],
            (CaptureTarget::Continuation, S::MultiNonzeroMembers) => &[
                S::WrongKey,
                S::WrongScript,
                S::WrongVersion,
                S::DisplacedSuccessor,
                S::ChangedMember,
            ],
            (CaptureTarget::Commitment, S::MultiNonzeroMembers) => {
                &[S::WrongParity, S::UncommittedLeaf]
            }
            _ => &[],
        };
        for mutant in subjects {
            let mut tx: elements::Transaction =
                elements::encode::deserialize(control.bytes()).unwrap();
            let key = *control.frozen().key();
            let member = control
                .frozen()
                .inputs()
                .iter()
                .position(FinalInput::member)
                .unwrap();
            match mutant {
                S::LaterBorn => tx.input[member].previous_output.vout = 1,
                S::Flagged => tx.input[member].is_pegin = true,
                S::WrongKey => {
                    let other = if key == [42; 32] {
                        vectors::keyed_final_manifest::SECOND_KEY
                    } else {
                        [42; 32]
                    };
                    let target = target_elements::reviewed_elements_tapscript().unwrap();
                    let commitment =
                        commit_keyed_bundle(&target, &prior::linked(other, false)).unwrap();
                    tx.output[0].script_pubkey =
                        elements::Script::from(commitment.output_program().to_vec());
                }
                S::WrongScript => {
                    let mut program = tx.output[0].script_pubkey.as_bytes().to_vec();
                    program[2] ^= 1;
                    tx.output[0].script_pubkey = elements::Script::from(program);
                }
                S::WrongVersion => {
                    let mut program = tx.output[0].script_pubkey.as_bytes().to_vec();
                    program[0] = 0x52;
                    tx.output[0].script_pubkey = elements::Script::from(program);
                }
                S::DisplacedSuccessor => tx.output.swap(0, 1),
                S::ChangedMember => {
                    let raw = self.foreign.as_ref().unwrap();
                    let foreign: elements::Transaction =
                        elements::encode::deserialize(raw).unwrap();
                    tx.input[member].previous_output = elements::OutPoint::new(foreign.txid(), 0);
                    let other = if key == [42; 32] {
                        vectors::keyed_final_manifest::SECOND_KEY
                    } else {
                        [42; 32]
                    };
                    let target = target_elements::reviewed_elements_tapscript().unwrap();
                    let commitment =
                        commit_keyed_bundle(&target, &prior::linked(other, false)).unwrap();
                    tx.input[member].witness.script_witness =
                        vec![commitment.leaf().to_vec(), commitment.control().to_vec()].into();
                }
                S::WrongParity | S::UncommittedLeaf => {
                    let mut witness: Vec<Vec<u8>> = tx.input[member]
                        .witness
                        .script_witness
                        .iter()
                        .map(<[u8]>::to_vec)
                        .collect();
                    if *mutant == S::WrongParity {
                        witness[1][0] ^= 1;
                    } else {
                        witness[0][1] ^= 1;
                    }
                    tx.input[member].witness.script_witness = witness.into();
                }
                S::KeyPathAttempt => {
                    tx.input[member].witness.script_witness = vec![vec![0; 64]].into();
                }
                _ => unreachable!("closed mutant census"),
            }
            // Flags are rejected at transaction form; the first-party digest
            // deliberately refuses this envelope, so no signature is invented.
            if !matches!(
                mutant,
                S::Flagged | S::WrongParity | S::UncommittedLeaf | S::KeyPathAttempt
            ) {
                self.resign(control, &mut tx);
            }
            self.raw_submission(key, *mutant, elements::encode::serialize(&tx));
        }
    }
}
