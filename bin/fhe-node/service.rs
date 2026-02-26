use fhestate_rs::constants::POLL_INTERVAL_SECS;
use fhestate_rs::{activate_server_key, load_server_key, LocalCache};

#[path = "net.rs"]
mod net;
use net::ChainListener;


use tokio::time::{sleep, Duration};
use log::{info, warn, error};
use sha2::Digest;
use std::sync::{Arc, Mutex};
use std::collections::{VecDeque, HashMap};
use std::error::Error;
use std::fs::File;
use std::path::Path;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::instruction::{Instruction, AccountMeta};
use solana_sdk::transaction::Transaction;
use std::str::FromStr;
use solana_transaction_status::UiTransactionEncoding;
use hex;

/// Minimum byte length of a serialised Task Anchor account.
const TASK_MIN_LEN: usize = 150; 

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    RevealRequested,
    Revealed,
    Challenged,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FheTask {
    pub account: Pubkey,
    pub id: u64,
    pub submitter: Pubkey,
    pub target_owner: Pubkey,
    pub operation: u8,
    pub input_uri: String,
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
    processed_states: Arc<Mutex<HashMap<Pubkey, u64>>>,
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
            processed_states: Arc::new(Mutex::new(HashMap::new())),
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
        // 1. Traditional Task Account Polling
        let accounts = self.listener.get_program_accounts(&self.program_id)?;
        for (pubkey, data) in accounts {
            let data: Vec<u8> = data;
            if data.len() < TASK_MIN_LEN { continue; }
            
            let mut disc_hasher = sha2::Sha256::new();
            disc_hasher.update(b"account:Task");
            let disc = &disc_hasher.finalize()[..8];
            if data[..8] != *disc { continue; }

            let id = u64::from_le_bytes(data[8..16].try_into().unwrap());
            let submitter = Pubkey::new_from_array(data[16..48].try_into().unwrap());
            let target_owner = Pubkey::new_from_array(data[48..80].try_into().unwrap());
            
            let uri_len = u32::from_le_bytes(data[112..116].try_into().unwrap()) as usize;
            if data.len() < 116 + uri_len { continue; }
            let input_uri = String::from_utf8_lossy(&data[116..116 + uri_len]).to_string();
            
            let op_offset = 116 + uri_len;
            let op = data[op_offset];
            let status_byte = data[op_offset + 1];

            if status_byte == 0 || status_byte == 4 { // Pending or RevealRequested
                let status = if status_byte == 0 { TaskStatus::Pending } else { TaskStatus::RevealRequested };
                let mut queue = self.task_queue.lock().unwrap();
                if !queue.iter().any(|t| t.account == pubkey) {
                    queue.push_back(FheTask {
                        account: pubkey,
                        id,
                        submitter,
                        target_owner,
                        operation: op,
                        input_uri,
                        status,
                    });
                    info!("   Task Detected: #{} status {:?} at {}", id, status, pubkey);
                }
            }
        }

        // 2. StateContainer (Inline) Polling
        let states = self.listener.get_state_containers(&self.program_id)?;
        for (pubkey, data) in states {
            if data.len() < 8 + 32 + 32 + 4 + 8 { continue; }
            
            let owner = Pubkey::new_from_array(data[8..40].try_into().unwrap());
            let uri_len = u32::from_le_bytes(data[72..76].try_into().unwrap()) as usize;
            let version_offset = 76 + uri_len;
            if data.len() < version_offset + 8 { continue; }
            let version = u64::from_le_bytes(data[version_offset..version_offset+8].try_into().unwrap());

            let mut processed = self.processed_states.lock().unwrap();
            let last_version = *processed.get(&pubkey).unwrap_or(&0);

            if version > last_version {
                info!("   Inline State Update Detected for PDA {} (v{} > v{})", pubkey, version, last_version);
                
                // Fetch the transaction to get the input data and operation
                let sigs = self.listener.get_client().get_signatures_for_address(&pubkey)?;
                if let Some(sig_info) = sigs.first() {
                    let sig = solana_sdk::signature::Signature::from_str(&sig_info.signature)?;
                    let tx_resp = self.listener.get_client().get_transaction(&sig, UiTransactionEncoding::Base64)?;
                    
                    let mut input_uri = String::new();
                    let mut op = 0;

                    if let Some(meta) = tx_resp.transaction.meta {
                        if meta.err.is_none() {
                            let mut disc_hasher = sha2::Sha256::new();
                            disc_hasher.update(b"global:submit_input");
                            let target_disc = &disc_hasher.finalize()[..8];

                            if let solana_transaction_status::EncodedTransaction::Binary(bin_tx, _) = tx_resp.transaction.transaction {
                                use base64::{Engine as _, engine::general_purpose};
                                let decoded_tx_bytes = general_purpose::STANDARD.decode(bin_tx.to_string()).unwrap_or_default();
                                if let Ok(tx) = bincode::deserialize::<solana_sdk::transaction::Transaction>(&decoded_tx_bytes) {
                                    for ix in &tx.message.instructions {
                                        if ix.data.len() >= 8 && &ix.data[..8] == target_disc {
                                            if let Some(&last_byte) = ix.data.last() {
                                                op = last_byte;
                                                info!("   Extracted Op Code: {} from transaction {}", op, sig);
                                            }
                                            break;
                                        }
                                    }
                                }
                            }

                            let hash_hex = hex::encode(&data[40..72]);
                            input_uri = format!("inline://{}", hash_hex);
                        }
                    }
                    
                    if !input_uri.is_empty() {
                        let mut queue = self.task_queue.lock().unwrap();
                        queue.push_back(FheTask {
                            account: Pubkey::default(),
                            id: version,
                            submitter: owner,
                            target_owner: owner, 
                            operation: op, 
                            input_uri,
                            status: TaskStatus::Pending,
                        });
                    }
                }
                processed.insert(pubkey, version);
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
            info!("Processing Task #{} (Op: {})", task.id, task.operation);

            // 1. Fetch input ciphertext from cache (Local, IPFS, or INLINE)
            let input_bytes: Vec<u8> = if task.input_uri.starts_with("inline://") {
                info!("   Task #{} resolving inline ciphertext from local cache...", task.id);
                let local_uri = task.input_uri.replace("inline://", "local://");
                match self.cache.load(&local_uri) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        error!("   Task #{} error: failed to load inline ciphertext: {}", task.id, e);
                        return Ok(());
                    }
                }
            } else {
                // Fallback to direct load or fetch if resolve is having visibility issues
                if task.input_uri.starts_with("local://") {
                    match self.cache.load(&task.input_uri) {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            error!("   Task #{} error: failed to load input ciphertext: {}", task.id, e);
                            return Ok(());
                        }
                    }
                } else {
                    // Simulating ipfs fetch for proof-of-concept
                    match self.cache.load(&task.input_uri.replace("ipfs://", "local://")) {
                        Ok(bytes) => bytes,
                        Err(_) => {
                            error!("   Task #{} error: IPFS simulation failed for {}", task.id, task.input_uri);
                            return Ok(());
                        }
                    }
                }
            };

            // Handle Reveal Logic
            if task.status == TaskStatus::RevealRequested {
                info!("   Task #{} is a Reveal Request. Generating decryption share...", task.id);
                let reveal_data = format!("REVEALED:StateHash:{}", hex::encode(&input_bytes));
                
                let mut disc_hasher = sha2::Sha256::new();
                disc_hasher.update(b"global:provide_reveal");
                let disc_hash = disc_hasher.finalize();
                
                let mut data = disc_hash[..8].to_vec();
                let reveal_bytes = reveal_data.as_bytes();
                data.extend_from_slice(&(reveal_bytes.len() as u32).to_le_bytes());
                data.extend_from_slice(reveal_bytes);

                let ix = Instruction::new_with_bytes(
                    self.program_id,
                    &data,
                    vec![
                        AccountMeta::new(task.account, false),
                        AccountMeta::new(self.keypair.pubkey(), true),
                    ],
                );
                
                self.send_tx(vec![ix]).await?;
                return Ok(());
            }


            // 2. Resolve StateContainer PDA for the user to find the *old* state
            let (state_pda, _bump) = Pubkey::find_program_address(
                &[b"state", task.target_owner.as_ref()],
                &self.program_id
            );

            let old_state_uri = match self.listener.get_client().get_account_data(&state_pda) {
                Ok(data) if data.len() >= 76 => {
                    let uri_len = u32::from_le_bytes(data[72..76].try_into().unwrap()) as usize;
                    if uri_len > 0 && data.len() >= 76 + uri_len {
                        Some(String::from_utf8_lossy(&data[76..76 + uri_len]).to_string())
                    } else {
                        None
                    }
                }
                _ => None,
            };

            // 3. Apply FHE state transition
            let (new_uri, result_hash) = match fhestate_rs::StateTransition::apply(
                &self.cache,
                old_state_uri.as_deref(),
                &input_bytes,
                task.operation,
            ) {
                Ok(res) => res,
                Err(e) => {
                    error!("   Task #{} FHE error: {}", task.id, e);
                    return Ok(());
                }
            };

            // Fetch current state hash for deterministic chaining
            let previous_state_hash: [u8; 32] = match self.listener.get_client().get_account_data(&state_pda) {
                Ok(data) if data.len() >= 72 => {
                    data[40..72].try_into().unwrap()
                }
                _ => [0u8; 32],
            };

            info!("   FHE Computation Success. New State: {}", new_uri);

            let mut discriminator_hasher = sha2::Sha256::new();
            let is_inline = task.account == Pubkey::default();

            // Fetch executor account address (we are the executor)
            let mut executor_account = Pubkey::default();
            if let Ok(accounts_data) = self.listener.get_program_accounts(&self.program_id) {
                let mut exec_disc_hasher = sha2::Sha256::new();
                exec_disc_hasher.update(b"account:Executor");
                let exec_disc = &exec_disc_hasher.finalize()[..8];
                
                for (pk, data) in accounts_data {
                    if data.len() >= 8 + 32 && &data[..8] == exec_disc && data[8..40] == self.keypair.pubkey().to_bytes() {
                        executor_account = pk;
                        break;
                    }
                }
            }

            let (_ix_name, accounts) = if is_inline {
                discriminator_hasher.update(b"global:update_state_pda");
                ("update_state_pda", vec![
                    AccountMeta::new(state_pda, false),
                    AccountMeta::new_readonly(task.submitter, false), 
                    AccountMeta::new(executor_account, false),
                    AccountMeta::new(self.keypair.pubkey(), true),
                ])
            } else {
                discriminator_hasher.update(b"global:update_state");
                ("update_state", vec![
                    AccountMeta::new(task.account, false),
                    AccountMeta::new(executor_account, false),
                    AccountMeta::new(state_pda, false),
                    AccountMeta::new(self.keypair.pubkey(), true),
                ])
            };

            let disc_hash = discriminator_hasher.finalize();
            let mut data = disc_hash[..8].to_vec();
            data.extend_from_slice(&previous_state_hash);
            data.extend_from_slice(&result_hash);
            
            let uri_bytes = new_uri.as_bytes();
            data.extend_from_slice(&(uri_bytes.len() as u32).to_le_bytes());
            data.extend_from_slice(uri_bytes);

            let ix = Instruction::new_with_bytes(
                self.program_id,
                &data,
                accounts,
            );



            match self.send_tx(vec![ix]).await {
                Ok(_) => info!("   Task #{} Completed!", task.id),
                Err(e) => error!("   Task #{} Failed: {}", task.id, e),
            }
        }

        Ok(())
    }

    async fn send_tx(&self, ixs: Vec<Instruction>) -> Result<String, Box<dyn Error>> {
        let rpc = self.listener.get_client();
        let blockhash = rpc.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &ixs,
            Some(&self.keypair.pubkey()),
            &[&self.keypair],
            blockhash,
        );
        let sig = rpc.send_and_confirm_transaction(&tx)?;
        info!("   Transaction Success: {}", sig);
        Ok(sig.to_string())
    }
}
