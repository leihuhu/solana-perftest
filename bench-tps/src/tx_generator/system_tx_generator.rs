use solana_sdk::{
    instruction::Instruction, signature::Keypair, signer::Signer, system_instruction, timing::timestamp, transaction::Transaction
};
use super::TxGenerator;

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
        _payer: &Keypair,
        _blockhash: &solana_sdk::hash::Hash,
        _args: std::collections::HashMap<String, String>,
    ) -> (Self, Vec<Transaction>)  {
        (self, Vec::new())
    }

    fn generate(
        &self, 
        keypairs: &Vec<Keypair>, 
        blockhash: &solana_sdk::hash::Hash,
    ) -> Vec<(Transaction, Option<u64>)> {
        keypairs.chunks(self.receiver_num + 1).map(|chunk| {
            let from = &chunk[0];
            let to_list = &chunk[1..];
            let instructions: Vec<Instruction> = to_list.iter().map(|to| {
                system_instruction::transfer(
                    &from.pubkey(),
                    &to.pubkey(),
                    1 // 转账1 lamport
                )
            }).collect();
            let mut tx = Transaction::new_with_payer(
                &instructions,
                Some(&from.pubkey())
            );
            tx.sign(&[from], *blockhash);
            (tx, Some(timestamp()))
        }).collect()
    }
}
