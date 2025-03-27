use log::*;
use rand::{thread_rng, Rng};
use solana_sdk::{
    hash::Hash, instruction::{AccountMeta, Instruction}, pubkey::Pubkey, signature::Keypair, signer::Signer, timing::timestamp, transaction::Transaction
};
use crate::tx_generator::TxGenerator;
use std::{collections::HashMap, convert::TryFrom};

use super::TimestampedTransaction;

const PROGRAM_ID: &str = "95HvDrwhVst6NHgypA77UfSxZQ33YAaCRjv9gZoi725n";


#[derive(Debug)]
pub struct BallotTxGenerator {
    program_id: Pubkey,
    ballot_boxes: Vec<AccountMeta>,
}

impl BallotTxGenerator {
    pub fn new() -> Self {
        Self {
            program_id: Pubkey::try_from(PROGRAM_ID).unwrap(),
            ballot_boxes: Vec::new(),
        }
    }
}

impl TxGenerator for BallotTxGenerator {    
    fn initialize(
        mut self,
        payer: &Keypair,
        blockhash: &Hash,
        args: HashMap<String, String>,
    ) -> (Self, Vec<Transaction>)  {
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
                        ]),    // Instruction data
                vec![
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new(ballot_box, false),
                    AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
                ], // Accounts needed
            );
            let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
            transaction.sign(&[&payer], *blockhash);
            transaction
        }).collect();
        (self, transactions)
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
        keypairs.iter().map(|voter| {
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
            let mut transaction = Transaction::new_with_payer(&[instruction], Some(&voter.pubkey()));
            transaction.sign(&[&voter], *blockhash);
            (transaction, Some(timestamp()))
        }).collect()
    }

}
