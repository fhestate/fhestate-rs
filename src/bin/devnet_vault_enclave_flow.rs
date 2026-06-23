use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
    instruction::{AccountMeta, Instruction},
};
use solana_client::rpc_client::RpcClient;
use std::str::FromStr;
use std::fs::File;
use std::error::Error;
use sha2::{Digest, Sha256};
use tfhe::{FheUint32, prelude::*};
use fhestate_rs::keys::{load_client_key, load_server_key, activate_server_key};

fn get_discriminator(name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("global:{}", name).as_bytes());
    let result = hasher.finalize();
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&result[..8]);
    discriminator
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("======================================================");
    println!("🛡️  FHESTATE Shielded Vault — TEE Enclave Devnet Flow");
    println!("======================================================");

    // 1. Establish RPC connection to Solana Devnet
    let rpc_url = "https://api.devnet.solana.com".to_string();
    println!("Connecting to Solana Devnet RPC: {}", rpc_url);
    let rpc = RpcClient::new(rpc_url);

    // 2. Load admin/authority keypair
    let wallet_path = "deploy-wallet.json";
    println!("Loading admin authority keypair from '{}'...", wallet_path);
    let file = File::open(wallet_path)?;
    let bytes: Vec<u8> = serde_json::from_reader(file)?;
    let admin = Keypair::from_bytes(&bytes)?;
    println!("Admin Public Address: {}", admin.pubkey());

    // 3. Define target program details
    let program_id = Pubkey::from_str("FuQzZCwPSRSVLT9gCgcft43a4RkapBJmSTC6CmdomeVQ")?;
    let (registry_pda, _) = Pubkey::find_program_address(&[b"vault_registry"], &program_id);
    let (vault_pda, _vault_bump) = Pubkey::find_program_address(&[b"vault_auth"], &program_id);
    let (enc_account, _) = Pubkey::find_program_address(&[b"enc_account", admin.pubkey().as_ref()], &program_id);

    println!("Vault Registry PDA: {}", registry_pda);
    println!("Vault Auth PDA: {}", vault_pda);
    println!("User Encrypted Account PDA: {}", enc_account);

    // Ensure registry exists on-chain
    if rpc.get_account(&registry_pda).is_err() {
        println!("Vault Registry PDA not found on-chain. Initializing it now...");
        let mut init_data = get_discriminator("initialize_vault").to_vec();
        init_data.extend_from_slice(&admin.pubkey().to_bytes());
        
        let init_ix = Instruction::new_with_bytes(
            program_id,
            &init_data,
            vec![
                AccountMeta::new(registry_pda, false),
                AccountMeta::new(admin.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );
        let blockhash = rpc.get_latest_blockhash()?;
        let mut tx_init = Transaction::new_with_payer(&[init_ix], Some(&admin.pubkey()));
        tx_init.sign(&[&admin], blockhash);
        let init_sig = rpc.send_and_confirm_transaction(&tx_init)?;
        println!("✅ Vault Registry PDA initialized on-chain. Tx: https://solscan.io/tx/{}?cluster=devnet", init_sig);
    }

    // 4. Generate keys and rotate Attestation Authority for remote attestation validation
    let attestation_authority = Keypair::new();
    println!("🔐 Rotating Attestation Authority on-chain to: {}", attestation_authority.pubkey());
    let mut att_disc = get_discriminator("update_attestation_authority").to_vec();
    att_disc.extend_from_slice(&attestation_authority.pubkey().to_bytes());
    
    let rotate_ix = Instruction::new_with_bytes(
        program_id,
        &att_disc,
        vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(registry_pda, false),
        ],
    );
    let blockhash = rpc.get_latest_blockhash()?;
    let mut tx_rotate = Transaction::new_with_payer(&[rotate_ix], Some(&admin.pubkey()));
    tx_rotate.sign(&[&admin], blockhash);
    let rotate_sig = rpc.send_and_confirm_transaction(&tx_rotate)?;
    println!("✅ Attestation Authority rotated. Tx: https://solscan.io/tx/{}?cluster=devnet", rotate_sig);

    // Align approved MRENCLAVE on-chain
    let mrenclave_hex = "a8f3b20c89de57f12e873111f930e12d4a5e6f3b0c8d7e6f9a0c1b2d3e4f5a6b";
    let mut mrenclave = [0u8; 32];
    hex::decode_to_slice(mrenclave_hex, &mut mrenclave)?;

    println!("Aligning approved MRENCLAVE on-chain to: {}", mrenclave_hex);
    let mut disc_mrenclave = get_discriminator("update_approved_mrenclave").to_vec();
    disc_mrenclave.extend_from_slice(&mrenclave);
    let ix_mrenclave = Instruction::new_with_bytes(
        program_id,
        &disc_mrenclave,
        vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(registry_pda, false),
        ],
    );
    let blockhash = rpc.get_latest_blockhash()?;
    let mut tx_mrenclave = Transaction::new_with_payer(&[ix_mrenclave], Some(&admin.pubkey()));
    tx_mrenclave.sign(&[&admin], blockhash);
    let sig_mrenclave = rpc.send_and_confirm_transaction(&tx_mrenclave)?;
    println!("✅ Approved MRENCLAVE aligned. Tx: https://solscan.io/tx/{}?cluster=devnet", sig_mrenclave);

    // 5. Generate and Register Ephemeral Enclave Signer (Secure Remote Attestation flow)
    println!("\n🔑 Booting ephemeral Enclave Signer in secure memory...");
    let enclave_signer = Keypair::new();
    let (enclave_pda, _) = Pubkey::find_program_address(&[b"enclave", enclave_signer.pubkey().as_ref()], &program_id);
    println!("Registering Enclave Signer PDA: {}", enclave_pda);

    let enclave_pubkey_bytes = enclave_signer.pubkey().to_bytes();
    let attestation_authority_bytes = attestation_authority.to_bytes();
    let dalek_keypair = ed25519_dalek::Keypair::from_bytes(&attestation_authority_bytes).unwrap();
    
    // 64-byte signed payload: [enclave_key (32 bytes) | mrenclave (32 bytes)]
    let mut message_payload = [0u8; 64];
    message_payload[..32].copy_from_slice(&enclave_pubkey_bytes);
    message_payload[32..64].copy_from_slice(&mrenclave);

    let ed25519_ix = solana_sdk::ed25519_instruction::new_ed25519_instruction(
        &dalek_keypair,
        &message_payload,
    );

    let mut disc = get_discriminator("register_enclave").to_vec();
    disc.extend_from_slice(&enclave_pubkey_bytes);

    let register_ix = Instruction::new_with_bytes(
        program_id,
        &disc,
        vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new_readonly(registry_pda, false),
            AccountMeta::new(enclave_pda, false),
            AccountMeta::new_readonly(solana_sdk::sysvar::instructions::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let blockhash = rpc.get_latest_blockhash()?;
    let mut tx_enclave = Transaction::new_with_payer(&[ed25519_ix, register_ix], Some(&admin.pubkey()));
    tx_enclave.sign(&[&admin], blockhash);
    let enclave_sig = rpc.send_and_confirm_transaction(&tx_enclave)?;
    println!("✅ Enclave Registered successfully. Tx: https://solscan.io/tx/{}?cluster=devnet", enclave_sig);

    // 6. Update Encrypted Daily Limit (FHE Math Operation)
    println!("\n[1/3] Loading FHE keys and encrypting Daily spending Limit...");
    let client_key = load_client_key("fhe_keys/client_key.bin")?;
    let server_key = load_server_key("fhe_keys/server_key.bin")?;
    activate_server_key(&server_key);

    let limit_amount: u32 = 10_000_000; // 0.01 SOL
    let limit_ct = FheUint32::encrypt(limit_amount, &client_key);
    let limit_ct_bytes = bincode::serialize(&limit_ct)?;
    
    // Compute SHA-256 hash of the large FHE daily limit ciphertext
    let mut limit_hasher = Sha256::new();
    limit_hasher.update(&limit_ct_bytes);
    let limit_hash = limit_hasher.finalize();

    // Store the 32-byte hash commitment in the [u8; 256] registry container (padded with zeros)
    let mut encrypted_daily_limit = [0u8; 256];
    encrypted_daily_limit[..32].copy_from_slice(&limit_hash);

    let mut limit_ix_data = get_discriminator("update_daily_limit").to_vec();
    limit_ix_data.extend_from_slice(&encrypted_daily_limit);


    let limit_ix = Instruction::new_with_bytes(
        program_id,
        &limit_ix_data,
        vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(registry_pda, false),
        ],
    );
    let blockhash = rpc.get_latest_blockhash()?;
    let mut tx_limit = Transaction::new_with_payer(&[limit_ix], Some(&admin.pubkey()));
    tx_limit.sign(&[&admin], blockhash);
    let limit_sig = rpc.send_and_confirm_transaction(&tx_limit)?;
    println!("✅ Encrypted Daily Limit set on-chain. Tx: https://solscan.io/tx/{}?cluster=devnet", limit_sig);

    // 7. Update Transaction Threshold
    println!("\n[2/3] Setting public transaction threshold alert limits...");
    let threshold_val: u64 = 2_500_000; // 0.0025 SOL
    let mut threshold_ix_data = get_discriminator("update_transaction_threshold").to_vec();
    threshold_ix_data.extend_from_slice(&threshold_val.to_le_bytes());

    let threshold_ix = Instruction::new_with_bytes(
        program_id,
        &threshold_ix_data,
        vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(registry_pda, false),
        ],
    );
    let blockhash = rpc.get_latest_blockhash()?;
    let mut tx_threshold = Transaction::new_with_payer(&[threshold_ix], Some(&admin.pubkey()));
    tx_threshold.sign(&[&admin], blockhash);
    let threshold_sig = rpc.send_and_confirm_transaction(&tx_threshold)?;
    println!("✅ Public Transaction Threshold set. Tx: https://solscan.io/tx/{}?cluster=devnet", threshold_sig);

    // 8. Execute Shielded Swap Proxy
    println!("\n[3/3] Executing Shielded Swap Proxy transaction on Devnet...");

    // Pre-fund vault PDA to make it rent-exempt
    let vault_balance = rpc.get_balance(&vault_pda)?;
    if vault_balance < 1_000_000 {
        println!("Vault PDA balance is low ({} lamports). Funding it with 0.002 SOL...", vault_balance);
        let fund_ix = solana_sdk::system_instruction::transfer(
            &admin.pubkey(),
            &vault_pda,
            2_000_000,
        );
        let blockhash = rpc.get_latest_blockhash()?;
        let mut tx_fund = Transaction::new_with_payer(&[fund_ix], Some(&admin.pubkey()));
        tx_fund.sign(&[&admin], blockhash);
        let fund_sig = rpc.send_and_confirm_transaction(&tx_fund)?;
        println!("✅ Vault PDA funded. Tx: https://solscan.io/tx/{}?cluster=devnet", fund_sig);
    }

    // Make sure user's encrypted account exists
    if rpc.get_account(&enc_account).is_err() {
        println!("Initializing user's encrypted account...");
        let disc = get_discriminator("initialize_account");
        let ix = Instruction::new_with_bytes(
            program_id,
            &disc,
            vec![
                AccountMeta::new(enc_account, false),
                AccountMeta::new(admin.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );
        let blockhash = rpc.get_latest_blockhash()?;
        let mut tx = Transaction::new_with_payer(&[ix], Some(&admin.pubkey()));
        tx.sign(&[&admin], blockhash);
        rpc.send_and_confirm_transaction(&tx)?;
        println!("User encrypted account initialized.");
    }

    // Homomorphic swap calculation
    let current_balance_ct = FheUint32::encrypt(500_000u32, &client_key);
    let received_ct = FheUint32::encrypt(48_000u32, &client_key);
    let new_balance_ct = &current_balance_ct + &received_ct;
    let new_balance_ct_bytes = bincode::serialize(&new_balance_ct)?;
    
    let mut hasher = Sha256::new();
    hasher.update(&new_balance_ct_bytes);
    let mut new_balance_hash = [0u8; 32];
    new_balance_hash.copy_from_slice(&hasher.finalize());

    let swap_in: u64 = 50_000;
    let swap_out_min: u64 = 45_000;

    let mut swap_ix_data = get_discriminator("shielded_swap_proxy").to_vec();
    swap_ix_data.extend_from_slice(&swap_in.to_le_bytes());
    swap_ix_data.extend_from_slice(&swap_out_min.to_le_bytes());
    swap_ix_data.extend_from_slice(&new_balance_hash);

    let swap_ix = Instruction::new_with_bytes(
        program_id,
        &swap_ix_data,
        vec![
            AccountMeta::new_readonly(enclave_signer.pubkey(), true),
            AccountMeta::new_readonly(enclave_pda, false),
            AccountMeta::new(registry_pda, false),
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(enc_account, false),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let blockhash = rpc.get_latest_blockhash()?;
    let mut tx_swap = Transaction::new_with_payer(&[swap_ix], Some(&admin.pubkey()));
    tx_swap.sign(&[&admin, &enclave_signer], blockhash);
    let swap_sig = rpc.send_and_confirm_transaction(&tx_swap)?;
    println!("✅ Shielded Swap Proxy executed successfully. Tx: https://solscan.io/tx/{}?cluster=devnet", swap_sig);

    println!("\n======================================================");
    println!("🎉 Shielded Vault TEE enclave flow complete on Devnet.");
    println!("======================================================");

    Ok(())
}
