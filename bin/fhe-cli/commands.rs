use fhestate_rs::KeyManager;
use log::{info, warn};
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use tfhe::prelude::*;
use fhestate_rs::LocalCache;
use sha2::{Digest, Sha256};



pub fn setup(
    rpc_url: &str,
    program_id: &str,
    wallet_path: &str,
) -> Result<(), Box<dyn Error>> {
    let prog_id = Pubkey::from_str(program_id)?;
    let is_memo = program_id == "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";

    info!("Starting FHEstate One-Time Setup...");

    // 0. Auto-Generate FHE Keys if missing
    if !fhestate_rs::keys::keys_exist("fhe_keys") {
        info!("   No FHE keys found. Generating new keyset (30-60s)...");
        let km = KeyManager::generate().map_err(|e| format!("Key generation failed: {}", e))?;
        km.save("fhe_keys").map_err(|e| format!("Failed to save keys: {}", e))?;
        info!("   Keys saved to 'fhe_keys/'");
    }

    if is_memo {
        info!("   Memo Program detected. Skipping on-chain initialization (not needed).");
        let mut config_file = File::create(".fhestate_registry")?;
        write!(config_file, "MEMO_MODE")?;
        info!("   Success! CLI configured for Memo-based Demo.");
        return Ok(());
    }

    // 1. Initialize Registry (Coordinator Mode)
    let wallet_file = File::open(wallet_path)?;
    let wallet_bytes: Vec<u8> = serde_json::from_reader(wallet_file)?;
    let payer = Keypair::from_bytes(&wallet_bytes)?;
    let rpc = RpcClient::new(rpc_url.to_string());

    let registry_keypair = Keypair::new();
    let registry_pubkey = registry_keypair.pubkey();
    
    let mut disc_hasher = Sha256::new();
    disc_hasher.update(b"global:initialize");
    let disc = disc_hasher.finalize();
    let mut data = disc[..8].to_vec();
    data.extend_from_slice(&100_000_000u64.to_le_bytes()); 

    let ix_reg = Instruction::new_with_bytes(
        prog_id,
        &data,
        vec![
            AccountMeta::new(registry_pubkey, true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
    );

    // 2. Initialize State
    let (state_pda, _bump) = Pubkey::find_program_address(&[b"state", payer.pubkey().as_ref()], &prog_id);
    
    let mut state_disc_hasher = Sha256::new();
    state_disc_hasher.update(b"global:initialize_state");
    let state_disc = state_disc_hasher.finalize();
    
    let ix_state = Instruction::new_with_bytes(
        prog_id,
        &state_disc[..8],
        vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
    );

    info!("   Sending combined setup transaction...");
    let signature = rpc.send_and_confirm_transaction(&Transaction::new_signed_with_payer(
        &[ix_reg, ix_state],
        Some(&payer.pubkey()),
        &[&payer, &registry_keypair],
        rpc.get_latest_blockhash()?,
    ))?;

    // 3. Save Registry Address
    let mut config_file = File::create(".fhestate_registry")?;
    write!(config_file, "{}", registry_pubkey)?;

    info!("   Success! Coordinator system is ready.");
    info!("   Registry: {}", registry_pubkey);
    info!("   Transaction: {}", signature);
    Ok(())
}

pub fn submit_task(
    rpc_url: &str,
    program_id: &str,
    wallet_path: &str,
    op: u8,
    value: u32,
) -> Result<(), Box<dyn Error>> {
    info!("Submitting FHE Task...");

    let prog_id = Pubkey::from_str(program_id)?;
    let is_memo = program_id == "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";

    // 1. Check/Load Registry ONLY if not in Memo Mode
    let registry_pubkey = if !is_memo {
        let registry_addr_str = std::fs::read_to_string(".fhestate_registry")
            .map_err(|_| "Registry not found. Did you run 'setup'?")?;
        if registry_addr_str == "MEMO_MODE" {
            return Err("CLI configured for Memo. Run 'setup --program <ID>' for Coordinator mode.".into());
        }
        Some(Pubkey::from_str(registry_addr_str.trim())?)
    } else {
        None
    };

    let wallet_file = File::open(wallet_path)?;
    let wallet_bytes: Vec<u8> = serde_json::from_reader(wallet_file)?;
    let payer = Keypair::from_bytes(&wallet_bytes)?;
    let rpc = RpcClient::new(rpc_url.to_string());

    // 2. Encryption
    use tfhe::ClientKey;
    let client_key_bytes = std::fs::read("fhe_keys/client_key.bin")?;
    let client_key: ClientKey = bincode::deserialize(&client_key_bytes)?;

    info!("   Encrypting value {}...", value);
    let encrypted_input = tfhe::FheUint32::encrypt(value, &client_key);
    let ciphertext_bytes = bincode::serialize(&encrypted_input)?;

    // 3. Cache management (always needed to get a URI)
    let cache = LocalCache::new(".fhe_cache");
    let uri = cache.store(&ciphertext_bytes)?;

    let task_keypair_opt = if is_memo {
        None
    } else {
        Some(Keypair::new())
    };

    let ix = if is_memo {
        info!("   Mode: Quick Demo (SPL Memo)");
        Instruction::new_with_bytes(
            prog_id,
            uri.as_bytes(),
            vec![AccountMeta::new_readonly(payer.pubkey(), true)],
        )
    } else {
        info!("   Mode: Coordinator (Full FHE)");
        let mut hasher = Sha256::new();
        hasher.update(&ciphertext_bytes);
        let input_hash: [u8; 32] = hasher.finalize().into();

        let mut disc_hasher = Sha256::new();
        disc_hasher.update(b"global:submit_task");
        let disc = disc_hasher.finalize();

        let mut data = disc[..8].to_vec();
        let id = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();
        data.extend_from_slice(&id.to_le_bytes());
        data.extend_from_slice(&input_hash);
        let uri_bytes = uri.as_bytes();
        data.extend_from_slice(&(uri_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(uri_bytes);
        data.push(op);
        
        if let Some(target_str) = target_owner {
            let target_pk = Pubkey::from_str(target_str)?;
            data.push(1); // Some
            data.extend_from_slice(target_pk.as_ref());
        } else {
            data.push(0); // None
        }

        Instruction::new_with_bytes(
            prog_id,
            &data,
            vec![
                AccountMeta::new(registry_pubkey.unwrap(), false),
                AccountMeta::new(task_keypair_opt.as_ref().unwrap().pubkey(), true),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            ],
        )
    };

    info!("   Sending transaction...");
    let recent_blockhash = rpc.get_latest_blockhash()?;
    
    let signature = if is_memo {
        rpc.send_and_confirm_transaction(&Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        ))?
    } else {
        let tkp = task_keypair_opt.as_ref().unwrap();
        rpc.send_and_confirm_transaction(&Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, tkp],
            recent_blockhash,
        ))?
    };

    info!("   Success! Task Submitted. Tx: {}", signature);
    Ok(())
}

pub fn reveal_task(
    rpc_url: &str,
    program_id: &str,
    wallet_path: &str,
    task_pubkey: &str,
) -> Result<(), Box<dyn Error>> {
    let rpc = RpcClient::new(rpc_url.to_string());
    let prog_id = Pubkey::from_str(program_id)?;
    let task_pk = Pubkey::from_str(task_pubkey)?;
    
    let wallet_file = File::open(wallet_path)?;
    let wallet_bytes: Vec<u8> = serde_json::from_reader(wallet_file)?;
    let payer = Keypair::from_bytes(&wallet_bytes)?;

    info!("Requesting Reveal for Task: {}", task_pk);

    let mut disc_hasher = Sha256::new();
    disc_hasher.update(b"global:request_reveal");
    let disc = disc_hasher.finalize();

    let ix = Instruction::new_with_bytes(
        prog_id,
        &disc[..8],
        vec![
            AccountMeta::new(task_pk, false),
            AccountMeta::new(payer.pubkey(), true),
        ],
    );

    let recent_blockhash = rpc.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    let signature = rpc.send_and_confirm_transaction(&tx)?;
    info!("   Success! Reveal Requested. Tx: {}", signature);
    
    Ok(())
}

pub fn submit_input(
    rpc_url: &str,
    program_id: &str,
    wallet_path: &str,
    operation: u8,
    value: u32,
    target_owner: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let rpc = RpcClient::new(rpc_url.to_string());
    let prog_id = Pubkey::from_str(program_id)?;
    
    let wallet_file = File::open(wallet_path)?;
    let wallet_bytes: Vec<u8> = serde_json::from_reader(wallet_file)?;
    let payer = Keypair::from_bytes(&wallet_bytes)?;

    use std::fs;
    use std::path::Path;
    let client_key_path = Path::new("fhe_keys/client_key.bin");
    if !client_key_path.exists() {
        return Err("Client key not found. Run keygen first.".into());
    }
    let client_key_bytes = fs::read(client_key_path)?;
    let client_key: tfhe::ClientKey = bincode::deserialize(&client_key_bytes)?;

    info!("Encrypting small input (value: {}) for inline submission...", value);
    let ciphertext = tfhe::FheUint32::encrypt(value, &client_key);
    let encrypted_data = bincode::serialize(&ciphertext)?;

    if encrypted_data.len() > 1000 {
        warn!("   CAUTION: Ciphertext size ({} bytes) exceeds Solana single-tx limit (1232 bytes).", encrypted_data.len());
        warn!("   This instruction will likely fail on Devnet. Use 'submit' for off-chain storage.");
    }

    // Cache locally so we can resolve it via hash later (Content-Addressed Storage)
    let cache = LocalCache::default();
    let uri = cache.store(&encrypted_data)?;
    info!("   Input cached. URI: {}", uri);

    let mut disc_hasher = Sha256::new();
    disc_hasher.update(b"global:submit_input");
    let disc = disc_hasher.finalize();

    let mut data = disc[..8].to_vec();
    data.extend_from_slice(&(encrypted_data.len() as u32).to_le_bytes());
    data.extend_from_slice(&encrypted_data);
    data.push(operation);
    
    if let Some(target_str) = target_owner {
        let target_pk = Pubkey::from_str(target_str)?;
        data.push(1); // Some
        data.extend_from_slice(target_pk.as_ref());
    } else {
        data.push(0); // None
    }

    let target_pk = if let Some(target) = target_owner {
        Pubkey::from_str(target)?
    } else {
        payer.pubkey()
    };

    let (state_pda, _bump) = Pubkey::find_program_address(
        &[b"state", target_pk.as_ref()],
        &prog_id,
    );

    let ix = Instruction::new_with_bytes(
        prog_id,
        &data,
        vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new(payer.pubkey(), true),
        ],
    );

    let signature = rpc.send_and_confirm_transaction(&Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        rpc.get_latest_blockhash()?,
    ))?;

    info!("   Success! Inline Input Submitted. Tx: {}", signature);
    Ok(())
}

pub fn init_state(
    rpc_url: &str,
    program_id: &str,
    wallet_path: &str,
) -> Result<(), Box<dyn Error>> {
    info!("Initializing StateContainer on Solana");

    let wallet_file = File::open(wallet_path)?;
    let wallet_bytes: Vec<u8> = serde_json::from_reader(wallet_file)?;
    let payer = Keypair::from_bytes(&wallet_bytes)?;
    
    let prog_id = Pubkey::from_str(program_id)?;
    let rpc = RpcClient::new(rpc_url.to_string());

    let (state_pda, _bump) = Pubkey::find_program_address(&[b"state", payer.pubkey().as_ref()], &prog_id);
    info!("   StateContainer PDA: {}", state_pda);

    let mut disc_hasher = Sha256::new();
    disc_hasher.update(b"global:initialize_state");
    let disc = disc_hasher.finalize();
    let data = disc[..8].to_vec();

    let ix = Instruction::new_with_bytes(
        prog_id,
        &data,
        vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
    );

    let recent_blockhash = rpc.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    info!("   Sending Transaction...");
    let signature = rpc.send_and_confirm_transaction(&tx)?;
    info!("   Initialized! Transaction Hash: {}", signature);

    Ok(())
}


