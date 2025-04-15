use solana_sdk::{
    hash::Hash, 
    signature::Keypair, 
    transaction::Transaction
};
use std::collections::HashMap;
pub type TimestampedTransaction = (Transaction, Option<u64>);
pub trait TxGenerator {
    fn initialize(self, keypairs: &Vec<Keypair>, blockhash: &Hash, args: HashMap<String, String>) -> Self where Self: Sized;
    fn generate(&self, keypairs: &Vec<Keypair>, blockhash: &Hash) -> Vec<TimestampedTransaction>;
}

pub mod ballot_tx_generator;
pub mod token_tx_generator;
pub mod replay_tx_generator;
pub mod system_tx_generator;