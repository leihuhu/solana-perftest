use log::*;
use rand::{thread_rng, Rng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, hash::Hash, instruction::{AccountMeta, Instruction}, pubkey::Pubkey, signature::Keypair, signer::Signer, timing::timestamp, transaction::Transaction
};
use crate::{bench_tps_client::BenchTpsClient, tx_generator::TxGenerator};
use std::{collections::HashMap, convert::TryFrom, sync::Arc, thread::sleep, time::Duration};

use super::TimestampedTransaction;

const PROGRAM_ID: &str = "95HvDrwhVst6NHgypA77UfSxZQ33YAaCRjv9gZoi725n";
const BALLOT_TRANSACTION_LOADED_ACCOUNTS_DATA_SIZE: u32 = 500 * 1024;
const MAX_COMPUTE_UNITS: u32 = 5_000; 

pub struct BallotTxGenerator<T: ?Sized> {
    client: Arc<T>,
    program_id: Pubkey,
    ballot_boxes: Vec<AccountMeta>,
}

impl<T> BallotTxGenerator<T>
where
    T: 'static + BenchTpsClient + Send + Sync + ?Sized,
{
    pub fn new(client: Arc<T>) -> Self {
        Self {
            client,
            program_id: Pubkey::try_from(PROGRAM_ID).unwrap(),
            ballot_boxes: Vec::new(),
        }
    }
}

impl<T> TxGenerator for BallotTxGenerator<T> 
where
    T: 'static + BenchTpsClient + Send + Sync + ?Sized,
{
    fn initialize(
        mut self,
        keypairs: &Vec<Keypair>,
        blockhash: &Hash,
        args: HashMap<String, String>,
    ) -> Self {
        let payer = &keypairs[0];
        let ballot_box_num = args.get("ballot_box_num").unwrap().parse::<u64>().unwrap();
        let init_discriminator = (
            220u8, 59u8, 207u8, 236u8, 
            108u8, 250u8, 47u8, 100u8  // 来自 IDL 的 init discriminator
        );
        let transactions = (0..ballot_box_num).map(|_| {
            let mut rng = thread_rng();
            let seed: u32 = rng.gen();
            let (ballot_box, _) = Pubkey::find_program_address(
                &["TEST".as_bytes(), seed.to_string().as_bytes()],
                &self.program_id
            );
            info!("ballot_box: {:?}", ballot_box);
            self.ballot_boxes.push(AccountMeta::new(ballot_box, false));
            let instruction = Instruction::new_with_borsh(
                self.program_id,
                &(init_discriminator,
                    "TEST".to_string(), 
                    seed.to_string(), 
                    vec![
                        "Spring".to_string(), 
                        "Yarn".to_string(), 
                        "Combat".to_string()
                        ]
                    ),    // Instruction data
                vec![
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new(ballot_box, false),
                    AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
                ], // Accounts needed
            );
            let instructions = [
                instruction
            ];
            let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
            transaction.sign(&[&payer], *blockhash);
            transaction
        }).collect();
        self.client.send_batch(transactions).expect("initialize ballot");
        for ballot_box in &self.ballot_boxes {
            loop {
                match self.client.get_account(&ballot_box.pubkey) {
                    Ok(box_account) => {
                        info!("Initialized bollot box: {:?}", box_account);
                        break;
                    }
                    Err(err) => {
                        info!("Waiting box account initialize: {:?}", err);
                        sleep(Duration::from_secs(1));
                    }
                }
            }
        }
        
        self
    }


    fn generate(
        &self, 
        keypairs: &Vec<Keypair>, 
        blockhash: &Hash,
    ) -> Vec<TimestampedTransaction> {
        let vote_discriminator = (
            227u8, 110u8, 155u8, 23u8,
            136u8, 126u8, 172u8, 25u8  // 来自 IDL 的 vote discriminator
        );

        keypairs.par_iter().map(|voter| {
            let num = self.ballot_boxes.len();
            let random_index = rand::thread_rng().gen_range(0..num);
            let ballot_box = self.ballot_boxes[random_index].clone();
            let instruction = Instruction::new_with_borsh(
                self.program_id,
                &(
                    vote_discriminator, 
                    rand::thread_rng().gen_range(0..3) as u64
                ),    // Instruction data
                vec![ballot_box], // Accounts needed
            );
            let instructions = [
                ComputeBudgetInstruction::set_loaded_accounts_data_size_limit(
                    BALLOT_TRANSACTION_LOADED_ACCOUNTS_DATA_SIZE,
                ),
                ComputeBudgetInstruction::set_compute_unit_limit(MAX_COMPUTE_UNITS),
                instruction
            ];
            let mut transaction = Transaction::new_with_payer(&instructions, Some(&voter.pubkey()));
            transaction.sign(&[&voter], *blockhash);
            (transaction, Some(timestamp()))
        }).collect()
    }

}
