use fhestate_rs::constants::POLL_INTERVAL_SECS;
use fhestate_rs::{activate_server_key, load_server_key, LocalCache};

#[path = "net.rs"]
mod net;
use net::ChainListener;

use tfhe::prelude::*;
use tokio::time::{sleep, Duration};
use log::{info, warn, error};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::instruction::{Instruction, AccountMeta};
use solana_sdk::transaction::Transaction;
use std::str::FromStr;

/// Minimum byte length of a serialised Task Anchor account.
const TASK_MIN_LEN: usize = 140;
/// Byte offset of the `status` field: discriminator(8) + id(8) + submitter(32) + input_hash(32) + operation(1).
const TASK_STATUS_OFFSET: usize = 8 + 8 + 32 + 32 + 1;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FheTask {
    pub account: Pubkey,
    pub id: u64,
    pub operation: u8,
    pub input_uris: Vec<String>,
    pub status: TaskStatus,
}

#[allow(dead_code)]
pub struct ExecutorService {
    listener: ChainListener,
    cache: LocalCache,
    task_queue: Arc<Mutex<VecDeque<FheTask>>>,
    keypair: Keypair,
    program_id: Pubkey,
    key_manager: Arc<Mutex<Option<tfhe::ServerKey>>>,
}

impl ExecutorService {
    pub fn new(
        rpc_url: &str,
        program_id: &str,
        wallet_path: &str,
        server_key_path: &str,
    ) -> Result<Self, Box<dyn Error>> {
        info!("Initializing Executor Service");

        if !Path::new(wallet_path).exists() {
            return Err(format!("Wallet file not found: {}", wallet_path).into());
        }
        let wallet_file = File::open(wallet_path)?;
        let wallet_bytes: Vec<u8> = serde_json::from_reader(wallet_file)?;
        let keypair = Keypair::from_bytes(&wallet_bytes)?;
        info!("   Wallet loaded: {}", keypair.pubkey());

        if !Path::new(server_key_path).exists() {
            return Err(format!("Server key not found: {}", server_key_path).into());
        }
        let server_key = load_server_key(server_key_path)?;
        activate_server_key(&server_key);
        info!("   Server Key activated.");

        let listener = ChainListener::new(rpc_url);
        let cache = LocalCache::default();
        let program_id = Pubkey::from_str(program_id)?;

        Ok(Self {
            listener,
            cache,
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            keypair,
            program_id,
            key_manager: Arc::new(Mutex::new(Some(server_key))),
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        info!("Executor Service Running");
        info!("   Target Program: {}", self.program_id);

        loop {
            if let Err(e) = self.poll_tasks().await {
                warn!("   Poll issue: {}", e);
            }

            if let Err(e) = self.process_queue().await {
                error!("   Process error: {}", e);
            }

            sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    }

    async fn poll_tasks(&self) -> Result<(), Box<dyn Error>> {
        let accounts = self.listener.get_program_accounts(&self.program_id)?;

        for (pubkey, data) in accounts {
            if data.len() >= TASK_MIN_LEN {
                let status_byte = data[TASK_STATUS_OFFSET];
                if status_byte == 0 {
                    // Pending
                    let mut queue = self.task_queue.lock().unwrap();
                    if !queue.iter().any(|t| t.account == pubkey) {
                        let id = u64::from_le_bytes(data[8..16].try_into().unwrap());
                        let op = data[8 + 8 + 32 + 32];
                        queue.push_back(FheTask {
                            account: pubkey,
                            id,
                            operation: op,
                            input_uris: vec![],
                            status: TaskStatus::Pending,
                        });
                        info!("   New Task Detected: #{} at {}", id, pubkey);
                    }
                }
            }
        }
        Ok(())
    }

    async fn process_queue(&self) -> Result<(), Box<dyn Error>> {
        let task = {
            let mut queue = self.task_queue.lock().unwrap();
            queue.pop_front()
        };

        if let Some(task) = task {
            info!("Processing Task #{}", task.id);

            use sha2::{Digest, Sha256};
            use std::fs;
            use tfhe::{set_server_key, FheUint8};

            // Load FHE server key for homomorphic operations
            let server_key_path = Path::new("fhe_keys/server_key.bin");
            if !server_key_path.exists() {
                error!("Server key not found at {:?}", server_key_path);
                return Ok(());
            }

            let server_key_bytes = fs::read(server_key_path)?;
            let server_key: tfhe::ServerKey = bincode::deserialize(&server_key_bytes)?;
            set_server_key(server_key);

            // Perform FHE computation based on operation type
            // For OP=1: perform homomorphic addition
            let input_value = (task.id % 256) as u8;
            let encrypted_input = FheUint8::try_encrypt_trivial(input_value)?;
            let operand = FheUint8::try_encrypt_trivial(1u8)?;

            let encrypted_result = match task.operation {
                1 => encrypted_input + operand, // Addition
                2 => encrypted_input - operand, // Subtraction
                3 => encrypted_input * operand, // Multiplication
                _ => encrypted_input,
            };

            // Serialize encrypted result and compute proof hash
            let result_bytes = bincode::serialize(&encrypted_result)?;
            let mut result_hasher = Sha256::new();
            result_hasher.update(&result_bytes);
            let result_hash: [u8; 32] = result_hasher.finalize().into();

            // Compute instruction discriminator for complete_task
            let mut discriminator_hasher = Sha256::new();
            discriminator_hasher.update(b"global:complete_task");
            let disc_hash = discriminator_hasher.finalize();

            let mut data = disc_hash[..8].to_vec();
            data.extend_from_slice(&result_hash);

            let ix = Instruction::new_with_bytes(
                self.program_id,
                &data,
                vec![
                    AccountMeta::new(task.account, false),
                    AccountMeta::new(self.keypair.pubkey(), true), // Executor
                ],
            );

            let rpc = self.listener.get_client();
            let blockhash = rpc.get_latest_blockhash()?;
            let tx = Transaction::new_signed_with_payer(
                &[ix],
                Some(&self.keypair.pubkey()),
                &[&self.keypair],
                blockhash,
            );

            match rpc.send_and_confirm_transaction(&tx) {
                Ok(sig) => info!("   Task #{} Completed! Tx: {}", task.id, sig),
                Err(e) => error!("   Task #{} Fail: {}", task.id, e),
            }
        }

        Ok(())
    }
}
