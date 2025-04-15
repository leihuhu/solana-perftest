use rayon::iter::{IntoParallelIterator, ParallelIterator};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, signature::Keypair, signer::Signer, system_instruction, timing::timestamp, transaction::Transaction
};
use super::{TimestampedTransaction, TxGenerator};

const TRANSFER_TRANSACTION_LOADED_ACCOUNTS_DATA_SIZE: u32 = 30 * 1024;
const TRANSFER_TRANSACTION_COMPUTE_UNIT: u32 = 600;

pub struct SystemTxGenerator{
    receiver_num: usize,
}

impl SystemTxGenerator {
    pub fn new(receiver_num: usize) -> Self {
        Self {
            receiver_num,
        }
    }
}

impl TxGenerator for SystemTxGenerator {
    fn initialize(
        self,
        _keypairs: &Vec<Keypair>,
        _blockhash: &solana_sdk::hash::Hash,
        _args: std::collections::HashMap<String, String>,
    ) -> Self  {
        self
    }

    fn generate(
        &self, 
        keypairs: &Vec<Keypair>, 
        blockhash: &solana_sdk::hash::Hash,
    ) -> Vec<TimestampedTransaction> {
        let source_chunk = &keypairs[..keypairs.len()/2];
        let dest_chunk = &keypairs[keypairs.len()/2..];
        
        (0..(keypairs.len()/2 - self.receiver_num + 1))
            .into_par_iter()
            .map(|i| {
                let from = &source_chunk[i];
                let to_list = &dest_chunk[i..i + self.receiver_num];
                
                let mut instructions: Vec<Instruction> = to_list.iter()
                    .map(|to| {
                        system_instruction::transfer(&from.pubkey(), &to.pubkey(), 1)
                    })
                    .collect();
                    
                instructions.push(
                    ComputeBudgetInstruction::set_loaded_accounts_data_size_limit(
                        TRANSFER_TRANSACTION_LOADED_ACCOUNTS_DATA_SIZE*self.receiver_num as u32,
                    )
                );

                instructions.push(
                    ComputeBudgetInstruction::set_compute_unit_limit(TRANSFER_TRANSACTION_COMPUTE_UNIT*self.receiver_num as u32),
                );
                
                let mut tx = Transaction::new_with_payer(&instructions, Some(&from.pubkey()));
                tx.sign(&[from], *blockhash);
                (tx, Some(timestamp()))
            })
            .collect()
    }
}
