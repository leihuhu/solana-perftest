use solana_sdk::{
    hash::Hash, system_instruction, signature::Keypair, signer::Signer, timing::timestamp, transaction::Transaction
};
use crate::tx_generator::TxGenerator;
use std::{collections::HashMap, path::Path};
use csv::Reader;

use super::TimestampedTransaction;


#[derive(Debug)]
pub struct ReplayTxGenerator {
    from_list: Vec<usize>,
    to_list: Vec<usize>,
    value_list: Vec<usize>,
}

#[derive(Debug, serde::Deserialize)]
struct CsvTransaction {
    from: String,
    to: String,
    value: u64,
}

impl ReplayTxGenerator {
    pub fn new() -> Self {
        Self {
            from_list: Vec::new(),
            to_list: Vec::new(),
            value_list: Vec::new(),
        }
    }
}

impl TxGenerator for ReplayTxGenerator {    
    fn initialize(
        mut self,
        _payer: &Keypair,
        _blockhash: &Hash,
        args: HashMap<String, String>,
    ) -> (Self, Vec<Transaction>)  {
        let replay_tx_path = Path::new(args.get("replay_tx_path").unwrap());
        let mut csv_reader = Reader::from_path(replay_tx_path).unwrap();
        
        for result in csv_reader.deserialize() {
            let csv_tx: CsvTransaction = result.unwrap();
            self.from_list.push(csv_tx.from.parse::<usize>().unwrap());
            self.to_list.push(csv_tx.to.parse::<usize>().unwrap());
            self.value_list.push(csv_tx.value as usize);
        }
        (self, Vec::new())
    }


    fn generate(
        &self, 
        keypairs: &Vec<Keypair>, 
        blockhash: &Hash,
    ) -> Vec<TimestampedTransaction> {
        let mut transactions: Vec<TimestampedTransaction> = Vec::with_capacity(keypairs.len());
        for i in 0..self.from_list.len() {
            let sender = &keypairs[self.from_list[i]];
            let receiver = &keypairs[self.to_list[i]];
            let amount = self.value_list[i] as u64;
            let instruction = system_instruction::transfer(
                &sender.pubkey(),
                &receiver.pubkey(),
                amount // 转账1 lamport
            );
            
            let mut transaction = Transaction::new_with_payer(&[instruction], Some(&sender.pubkey()));
            transaction.sign(&[&sender], *blockhash);
            transactions.push((transaction, Some(timestamp())));
        };
        transactions
    }

}
