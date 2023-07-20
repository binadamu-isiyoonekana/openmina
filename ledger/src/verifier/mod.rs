use std::sync::Arc;

use crate::{
    proofs::{verifier_index::get_verifier_index, VerifierIndex, VerifierSRS},
    scan_state::{
        scan_state::transaction_snark::{
            LedgerProof, LedgerProofWithSokMessage, SokMessage, TransactionSnark,
        },
        transaction_logic::{valid, verifiable, WithStatus},
    },
};

use self::common::CheckResult;

#[derive(Debug, Clone)]
pub struct Verifier;

use once_cell::sync::Lazy;

// TODO: Move this into `Verifier` struct above
static VERIFIER_INDEX: Lazy<Arc<VerifierIndex>> = Lazy::new(|| {
    use crate::proofs::verifier_index::VerifierKind;
    Arc::new(get_verifier_index(VerifierKind::Transaction))
});

// TODO: Move this into `Verifier` struct above
static SRS: Lazy<Arc<VerifierSRS>> =
    Lazy::new(|| std::sync::Arc::new(crate::proofs::accumulator_check::get_srs()));

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/transaction_snark/transaction_snark.ml#L3492
fn verify(ts: Vec<(LedgerProof, SokMessage)>) -> Result<(), String> {
    if ts.iter().all(|(proof, msg)| {
        let LedgerProof(TransactionSnark { statement, .. }) = proof;
        statement.sok_digest == msg.digest()
    }) {
        let verifier_index = VERIFIER_INDEX.as_ref();
        let srs = SRS.as_ref();

        // for (proof, msg) in ts {
        //     let LedgerProof(TransactionSnark {
        //         statement,
        //         proof: p,
        //     }) = &proof;
        //     let (stmt, p) = (statement, &**p);
        //     if !crate::proofs::verification::verify_transaction([(stmt, p)], verifier_index) {
        //         let a: mina_p2p_messages::v2::LedgerProofProdStableV2 = (&proof).into();
        //         let b: mina_p2p_messages::v2::MinaBaseSokMessageStableV1 = (&msg).into();
        //         let mut file = std::fs::File::create("ledger_proof2.bin").unwrap();
        //         binprot::BinProtWrite::binprot_write(&a, &mut file).unwrap();
        //         file.sync_all().unwrap();
        //         let mut file = std::fs::File::create("sok_msg2.bin").unwrap();
        //         binprot::BinProtWrite::binprot_write(&b, &mut file).unwrap();
        //         file.sync_all().unwrap();
        //         panic!();
        //     }
        // }

        let proofs = ts.iter().map(|(proof, _)| {
            let LedgerProof(TransactionSnark { statement, proof }) = proof;
            (statement, &**proof)
        });

        if !crate::proofs::verification::verify_transaction(proofs, verifier_index, srs) {
            return Err("Transaction_snark.verify: verification failed".into());
        }
        Ok(())
    } else {
        Err("Transaction_snark.verify: Mismatched sok_message".into())
    }
}

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/verifier/dummy.ml#L59C1-L75C81
#[cfg(test)]
fn verify_digest_only(ts: Vec<(LedgerProof, SokMessage)>) -> Result<(), String> {
    use crate::scan_state::scan_state::transaction_snark::SokDigest;

    if ts.iter().all(|(proof, msg)| {
        let LedgerProof(TransactionSnark { statement, .. }) = proof;
        statement.sok_digest == SokDigest::default() || statement.sok_digest == msg.digest()
    }) {
        Ok(())
    } else {
        Err("Transaction_snark.verify: Mismatched sok_message".into())
    }
}

impl Verifier {
    pub fn verify(&self, _proofs: &[LedgerProofWithSokMessage]) -> Result<Result<(), ()>, String> {
        // Implement verification later
        //
        // https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/pickles/pickles.ml#L1122
        // https://viable-systems.slack.com/archives/D01SVA87PQC/p1671715846448749
        Ok(Ok(()))
    }

    /// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/verifier/prod.ml#L138
    #[allow(unreachable_code)]
    pub fn verify_transaction_snarks(
        &self,
        ts: Vec<(LedgerProof, SokMessage)>,
    ) -> Result<(), String> {
        #[cfg(test)]
        return verify_digest_only(ts);

        verify(ts)
    }

    pub fn verify_commands(
        &self,
        cmds: Vec<WithStatus<verifiable::UserCommand>>,
    ) -> Result<Vec<valid::UserCommand>, VerifierError> {
        // TODO

        let xs: Vec<_> = cmds
            .into_iter()
            .map(common::check)
            .map(|cmd| {
                match cmd {
                    common::CheckResult::Valid(cmd) => Ok(cmd),
                    e => Err(e)
                // common::CheckResult::ValidAssuming(_) => todo!(),
                // common::CheckResult::InvalidKeys(_) => todo!(),
                // common::CheckResult::InvalidSignature(_) => todo!(),
                // common::CheckResult::InvalidProof => todo!(),
                // common::CheckResult::MissingVerificationKey(_) => todo!(),
            }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(xs)
    }
}

#[derive(Debug, derive_more::From)]
pub enum VerifierError {
    CheckError(CheckResult),
}

pub mod common {
    use mina_signer::CompressedPubKey;

    use crate::scan_state::transaction_logic::{valid, verifiable, zkapp_command, WithStatus};

    #[derive(Debug)]
    pub enum CheckResult {
        Valid(valid::UserCommand),
        ValidAssuming((valid::UserCommand, Vec<()>)),
        InvalidKeys(Vec<CompressedPubKey>),
        InvalidSignature(Vec<CompressedPubKey>),
        InvalidProof,
        MissingVerificationKey(Vec<CompressedPubKey>),
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/verifier/common.ml#L29
    pub fn check(cmd: WithStatus<verifiable::UserCommand>) -> CheckResult {
        use verifiable::UserCommand::{SignedCommand, ZkAppCommand};

        // TODO: Implement

        match cmd.data {
            SignedCommand(cmd) => {
                if !cmd.check_valid_keys() {
                    let public_keys = cmd.public_keys().into_iter().cloned().collect();
                    return CheckResult::InvalidKeys(public_keys);
                }
                match verifiable::check_only_for_signature(cmd) {
                    Ok(cmd) => CheckResult::Valid(cmd),
                    Err(cmd) => CheckResult::InvalidSignature(
                        cmd.public_keys().into_iter().cloned().collect(),
                    ),
                }
            }
            ZkAppCommand(cmd) => {
                // TODO: Implement rest

                let zkapp_command = zkapp_command::valid::of_verifiable(*cmd);

                CheckResult::Valid(valid::UserCommand::ZkAppCommand(Box::new(zkapp_command)))

                // match  {
                //     Some(cmd) => {
                //         CheckResult::Valid(valid::UserCommand::ZkAppCommand(Box::new(cmd)))
                //     }
                //     None => CheckResult::InvalidProof, // TODO
                // }
            }
        }
    }
}
