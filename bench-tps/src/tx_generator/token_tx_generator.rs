use log::info;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, hash::Hash, instruction::{AccountMeta, Instruction}, pubkey::Pubkey, signature::Keypair, signer::Signer, timing::timestamp, transaction::Transaction
};
use crate::{bench_tps_client::BenchTpsClient, tx_generator::TxGenerator};
use std::{collections::HashMap, convert::TryFrom, sync::Arc, thread::sleep, time::Duration};

use super::TimestampedTransaction;

const PROGRAM_ID: &str = "59XDJqwb67HEYq779ZGwDdv5ZUCncqqGXAicnAxkb7U9";
const TRANSFER_TRANSACTION_LOADED_ACCOUNTS_DATA_SIZE: u32 = 200 * 1024;
const MAX_COMPUTE_UNITS: u32 = 3_000; 


#[derive(Debug)]
pub struct TokenTxGenerator<T: ?Sized> {
    client: Arc<T>,
    program_id: Pubkey,
    receiver_num: usize,
    token_accounts: HashMap<Pubkey, Pubkey>,
}

impl<T> TokenTxGenerator<T>
where
    T: 'static + BenchTpsClient + Send + Sync + ?Sized,
{
    pub fn new(client: Arc<T>, receiver_num: usize) -> Self {
        Self {
            client,
            program_id: Pubkey::try_from(PROGRAM_ID).unwrap(),
            receiver_num,
            token_accounts: HashMap::new(),
        }
    }
}

impl<T> TxGenerator for TokenTxGenerator<T>
where
    T: 'static + BenchTpsClient + Send + Sync + ?Sized,
{    
    fn initialize(
        mut self,
        keypairs: &Vec<Keypair>,
        blockhash: &Hash,
        _args: HashMap<String, String>,
    ) -> Self  {
        let init_user_discriminator = (
            14u8, 51u8, 68u8, 159u8,
            237u8, 78u8, 158u8, 102u8
        );
        
        // token accounts waiting to be created
        let token_pairs: Vec<_> = keypairs.par_iter().map(|pair| {
            let (token_account, _) = Pubkey::find_program_address(
                &["balance".as_bytes(), &pair.pubkey().to_bytes()],
                &self.program_id
            );
            (pair, token_account)
        }).collect();
        info!("total token accounts: {:?}", token_pairs.len());

        // filter out the token accounts that already exist
        let mut accounts_to_create: Vec<_> = token_pairs.par_iter()
            .filter(|(_, ta)| self.client.get_account(ta).is_err())
            .map(|pair| *pair)
            .collect();
        
        // insert the token accounts into the map
        for (pair, token_account) in &token_pairs {
            self.token_accounts.insert(pair.pubkey(), *token_account);
        }

        while accounts_to_create.len() > 0 {
            info!("accounts_to_create: {:?}", accounts_to_create.len());
            // create the token accounts in parallel
            let transactions = accounts_to_create.par_iter().map(|(pair, token_account)| {
                let instruction = Instruction::new_with_borsh(
                    self.program_id,
                    &(init_user_discriminator, 1000000000000000u64),
                    vec![
                        AccountMeta::new(*token_account, false),
                        AccountMeta::new(pair.pubkey(), true),
                        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
                    ],
                );
                let instructions = [
                    instruction
                ];
                let mut transaction = Transaction::new_with_payer(&instructions, Some(&pair.pubkey()));
                transaction.sign(&[pair], *blockhash);
                transaction
            }).collect();
            
            self.client.send_batch(transactions).expect("initialize token account");
            sleep(Duration::from_secs(10));
            accounts_to_create = accounts_to_create.par_iter()
                .filter(|(_, ta)| self.client.get_account(ta).is_err())
                .map(|pair| *pair)
                .collect();
        }

        self
    }


    fn generate(
        &self, 
        keypairs: &Vec<Keypair>, 
        blockhash: &Hash,
    ) -> Vec<TimestampedTransaction> {
        let transfer_discriminator = (
            163u8, 52u8, 200u8, 231u8,
            140u8, 3u8, 69u8, 186u8// transfer discriminator
        );

        let source_chunk = &keypairs[..keypairs.len()/2];
        let dest_chunk = &keypairs[keypairs.len()/2..];
        
        (0..(keypairs.len()/2 - self.receiver_num + 1))
            .into_par_iter()
            .map(|i| {
                let from = &source_chunk[i];
                let to_list = &dest_chunk[i..i + self.receiver_num];
                
                let mut instructions: Vec<Instruction> = to_list.iter()
                    .map(|to| {
                        let from_token_account = self.token_accounts.get(&from.pubkey()).unwrap();
                        let to_token_account = self.token_accounts.get(&to.pubkey()).unwrap();
                        Instruction::new_with_borsh(
                            self.program_id,
                            &(
                                transfer_discriminator, 
                                1u64
                            ),    // Instruction data
                            vec![
                                AccountMeta::new(*from_token_account, false),
                                AccountMeta::new(*to_token_account, false),
                                AccountMeta::new(from.pubkey(), true),
                            ], // Accounts needed
                        )
                    })
                    .collect();
                instructions.push(
                    ComputeBudgetInstruction::set_compute_unit_limit(MAX_COMPUTE_UNITS*self.receiver_num as u32),
                );
                instructions.push(
                    ComputeBudgetInstruction::set_loaded_accounts_data_size_limit(
                        TRANSFER_TRANSACTION_LOADED_ACCOUNTS_DATA_SIZE*self.receiver_num as u32,
                    )
                );
                
                let mut tx = Transaction::new_with_payer(&instructions, Some(&from.pubkey()));
                tx.sign(&[from], *blockhash);
                (tx, Some(timestamp()))
            })
            .collect()
    }

}
