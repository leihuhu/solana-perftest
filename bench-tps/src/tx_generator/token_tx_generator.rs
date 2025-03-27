use rand::Rng;
use solana_sdk::{
    hash::Hash, instruction::{AccountMeta, Instruction}, pubkey::Pubkey, signature::Keypair, signer::Signer, system_program, timing::timestamp, transaction::Transaction
};
use crate::tx_generator::TxGenerator;
use std::{collections::HashMap, convert::TryFrom};

use super::TimestampedTransaction;

const PROGRAM_ID: &str = "59XDJqwb67HEYq779ZGwDdv5ZUCncqqGXAicnAxkb7U9";


#[derive(Debug)]
pub struct TokenTxGenerator {
    program_id: Pubkey,
    receiver_num: usize,
}

impl TokenTxGenerator {
    pub fn new(receiver_num: usize) -> Self {
        Self {
            program_id: Pubkey::try_from(PROGRAM_ID).unwrap(),
            receiver_num,
        }
    }
}

impl TxGenerator for TokenTxGenerator {    
    fn initialize(
        self,
        _payer: &Keypair,
        _blockhash: &Hash,
        _args: HashMap<String, String>,
    ) -> (Self, Vec<Transaction>)  {
        (self, Vec::new())
    }


    fn generate(
        &self, 
        keypairs: &Vec<Keypair>, 
        blockhash: &Hash,
    ) -> Vec<TimestampedTransaction> {
        let mut transactions: Vec<TimestampedTransaction> = Vec::with_capacity(keypairs.len());
        let transfer_sol_with_cpi_discriminator = (
            209u8, 108u8, 135u8, 67u8, 
            87u8, 217u8, 209u8, 143u8 // 来自 IDL 的 transfer_sol_with_cpi discriminator
        );
        // for i in 0..keypairs.len() {
        //     let sender = &keypairs[i];
        //     let mut instructions = Vec::with_capacity(self.receiver_num);
        //     for _j in 0..self.receiver_num {
        //         let random_index = rand::thread_rng().gen_range(0..self.receiver_num);
        //         let receiver = &keypairs[random_index];
        //         let instruction = Instruction::new_with_borsh(
        //             self.program_id,
        //             &(
        //                 transfer_sol_with_program_discriminator, 
        //                 100u64
        //             ),    // Instruction data
        //             vec![
        //                 AccountMeta::new(sender.pubkey(), true),
        //                 AccountMeta::new(receiver.pubkey(), false),
        //             ], // Accounts needed
        //         );
        //         instructions.push(instruction);
        //     }
        //     let mut transaction = Transaction::new_with_payer(&instructions, Some(&sender.pubkey()));
        //     transaction.sign(&[&sender], *blockhash);
        //     transactions.push((transaction, Some(timestamp())));
        // };
        keypairs.chunks(2).for_each(|chunk| {
            let sender = &chunk[0];
            let receiver = &chunk[1];
            let instruction = Instruction::new_with_borsh(
                self.program_id,
                &(
                    transfer_sol_with_cpi_discriminator, 
                    1u64
                ),    // Instruction data
                vec![
                    AccountMeta::new(sender.pubkey(), true),
                    AccountMeta::new(receiver.pubkey(), false),
                    AccountMeta::new_readonly(system_program::id(), false),
                ], // Accounts needed
            );
            let mut transaction = Transaction::new_with_payer(&[instruction], Some(&sender.pubkey()));
            transaction.sign(&[&sender], *blockhash);
            transactions.push((transaction, Some(timestamp())));
        });
        transactions
    }

}
