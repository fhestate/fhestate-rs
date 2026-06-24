use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction, system_program,
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
    println!("=================================================");
    println!("🔐 FHESTATE SHIELDED VAULT —  DEVNET WORKFLOW");
    println!("=================================================");

    // 1. Initialize RPC connection
    let rpc_url = "https://api.devnet.solana.com".to_string();
    println!("Connecting to Solana Devnet: {}", rpc_url);
    let rpc = RpcClient::new(rpc_url);

    // 2. Load developer/authority keypair
    let wallet_path = "deploy-wallet.json";
    println!("Loading authority wallet from '{}'...", wallet_path);
    let file = File::open(wallet_path)?;
    let bytes: Vec<u8> = serde_json::from_reader(file)?;
    let payer = Keypair::from_bytes(&bytes)?;
    println!("Authority Address: {}", payer.pubkey());

    let balance = rpc.get_balance(&payer.pubkey())?;
    println!("Authority Balance: {:.6} SOL", balance as f64 / 1_000_000_000.0);

    // 3. Define Shielded Vault Program details
    let program_id = Pubkey::from_str("FuQzZCwPSRSVLT9gCgcft43a4RkapBJmSTC6CmdomeVQ")?;
    
    // Derive PDA addresses
    let (registry_pda, _registry_bump) = Pubkey::find_program_address(&[b"vault_registry"], &program_id);
    let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault_auth"], &program_id);
    let (enc_account_a, _) = Pubkey::find_program_address(&[b"enc_account", payer.pubkey().as_ref()], &program_id);

    // Create a new fresh receiver keypair
    let receiver_keypair = Keypair::new();
    let (enc_account_b, _) = Pubkey::find_program_address(&[b"enc_account", receiver_keypair.pubkey().as_ref()], &program_id);

    println!("\n--- Derived Addresses ---");
    println!("Program ID: {}", program_id);
    println!("Registry PDA: {}", registry_pda);
    println!("Vault PDA (holding SOL): {}", vault_pda);
    println!("Sender (A) Encrypted Account: {}", enc_account_a);
    println!("Receiver (B) Public Key: {}", receiver_keypair.pubkey());
    println!("Receiver (B) Encrypted Account: {}", enc_account_b);
    println!("-------------------------\n");

    // ----------------------------------------------------
    // PHASE 1: Initialize Accounts
    // ----------------------------------------------------
    println!("Checking if Vault Registry is initialized on-chain...");
    if rpc.get_account(&registry_pda).is_err() {
        println!("Registry PDA not initialized. Sending initialize_vault transaction...");
        let disc = get_discriminator("initialize_vault");
        let ix = Instruction::new_with_bytes(
            program_id,
            &disc,
            vec![
                AccountMeta::new(registry_pda, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );
        let blockhash = rpc.get_latest_blockhash()?;
        let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
        tx.sign(&[&payer], blockhash);
        let sig = rpc.send_and_confirm_transaction(&tx)?;
        println!("Registry initialized: https://solscan.io/tx/{}?cluster=devnet", sig);
    } else {
        println!("Vault Registry is already initialized.");
    }

    println!("Checking if Sender Encrypted Account is initialized...");
    if rpc.get_account(&enc_account_a).is_err() {
        println!("Sender encrypted account not initialized. Initializing...");
        let disc = get_discriminator("initialize_account");
        let ix = Instruction::new_with_bytes(
            program_id,
            &disc,
            vec![
                AccountMeta::new(enc_account_a, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );
        let blockhash = rpc.get_latest_blockhash()?;
        let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
        tx.sign(&[&payer], blockhash);
        let sig = rpc.send_and_confirm_transaction(&tx)?;
        println!("Sender account initialized: https://solscan.io/tx/{}?cluster=devnet", sig);
    } else {
        println!("Sender encrypted account is already initialized.");
    }

    println!("Checking if Receiver Encrypted Account is initialized...");
    if rpc.get_account(&enc_account_b).is_err() {
        println!("Receiver encrypted account not initialized. Funding and initializing...");
        
        // Fund the receiver wallet so it can sign the creation
        let fund_ix = system_instruction::transfer(&payer.pubkey(), &receiver_keypair.pubkey(), 50_000_000);
        let blockhash = rpc.get_latest_blockhash()?;
        let mut fund_tx = Transaction::new_with_payer(&[fund_ix], Some(&payer.pubkey()));
        fund_tx.sign(&[&payer], blockhash);
        let fund_sig = rpc.send_and_confirm_transaction(&fund_tx)?;
        println!("Funded Receiver: https://solscan.io/tx/{}?cluster=devnet", fund_sig);

        // Initialize receiver account
        let disc = get_discriminator("initialize_account");
        let ix = Instruction::new_with_bytes(
            program_id,
            &disc,
            vec![
                AccountMeta::new(enc_account_b, false),
                AccountMeta::new(receiver_keypair.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );
        let blockhash = rpc.get_latest_blockhash()?;
        let mut tx = Transaction::new_with_payer(&[ix], Some(&receiver_keypair.pubkey()));
        tx.sign(&[&receiver_keypair], blockhash);
        let sig = rpc.send_and_confirm_transaction(&tx)?;
        println!("Receiver account initialized: https://solscan.io/tx/{}?cluster=devnet", sig);
    } else {
        println!("Receiver encrypted account is already initialized.");
    }

    // ----------------------------------------------------
    // PHASE 2: Shield Funds (Deposit SOL into vault)
    // ----------------------------------------------------
    let shield_amount: u64 = 1_000_000; // 0.001 SOL
    println!("\n➡️  Shielding {} lamports (0.001 SOL) to Vault...", shield_amount);
    let mut disc = get_discriminator("shield_funds").to_vec();
    disc.extend_from_slice(&shield_amount.to_le_bytes());

    let ix = Instruction::new_with_bytes(
        program_id,
        &disc,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(registry_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    let blockhash = rpc.get_latest_blockhash()?;
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(&[&payer], blockhash);
    let shield_sig = rpc.send_and_confirm_transaction(&tx)?;
    println!("✅ Shield Tx: https://solscan.io/tx/{}?cluster=devnet", shield_sig);

    // ----------------------------------------------------
    // PHASE 3: Confidential Transfer 
    // ----------------------------------------------------
    println!("\n🔒 Loading FHE context and keys...");
    let client_key = load_client_key("fhe_keys/client_key.bin")?;
    let server_key = load_server_key("fhe_keys/server_key.bin")?;
    activate_server_key(&server_key);
    println!("Activated Server Key for homomorphic math.");

    println!("Encrypting starting balances and transfer amount...");
    let sender_start_bal = FheUint32::encrypt(0u32, &client_key);
    let deposit_amount_ct = FheUint32::encrypt(shield_amount as u32, &client_key);
    let receiver_start_bal = FheUint32::encrypt(0u32, &client_key);
    
    // Transfer amount: 300,000 lamports
    let transfer_amount: u32 = 300_000;
    let transfer_amount_ct = FheUint32::encrypt(transfer_amount, &client_key);

    println!("Executing off-chain homomorphic addition and subtraction on encrypted balances...");
    // Update sender balance: balance = balance + deposit - transfer
    let sender_bal_after_deposit = &sender_start_bal + &deposit_amount_ct;
    let sender_new_ct = &sender_bal_after_deposit - &transfer_amount_ct;

    // Update receiver balance: balance = balance + transfer
    let receiver_new_ct = &receiver_start_bal + &transfer_amount_ct;

    // Decrypt balances locally for verification (only possible by FHE worker with client key)
    let sender_decrypted: u32 = sender_new_ct.decrypt(&client_key);
    let receiver_decrypted: u32 = receiver_new_ct.decrypt(&client_key);
    println!("   [Verify] Sender New FHE Balance  : {} lamports (expected: 700000)", sender_decrypted);
    println!("   [Verify] Receiver New FHE Balance: {} lamports (expected: 300000)", receiver_decrypted);

    // Serialize ciphertexts
    let sender_ct_bytes = bincode::serialize(&sender_new_ct)?;
    let receiver_ct_bytes = bincode::serialize(&receiver_new_ct)?;

    // Compute cryptographic commitments
    let mut sender_hasher = Sha256::new();
    sender_hasher.update(&sender_ct_bytes);
    let mut sender_hash = [0u8; 32];
    sender_hash.copy_from_slice(&sender_hasher.finalize());

    let mut receiver_hasher = Sha256::new();
    receiver_hasher.update(&receiver_ct_bytes);
    let mut receiver_hash = [0u8; 32];
    receiver_hash.copy_from_slice(&receiver_hasher.finalize());

    println!("   Sender Commitment Hash: {:x?}", sender_hash);
    println!("   Receiver Commitment Hash: {:x?}", receiver_hash);

    println!("\n➡️  Posting FHE confidential transfer state to Solana Devnet...");
    let mut disc = get_discriminator("execute_transfer_fhe").to_vec();
    disc.extend_from_slice(&sender_hash);
    disc.extend_from_slice(&receiver_hash);

    let ix = Instruction::new_with_bytes(
        program_id,
        &disc,
        vec![
            AccountMeta::new_readonly(payer.pubkey(), true),
            AccountMeta::new_readonly(registry_pda, false),
            AccountMeta::new(enc_account_a, false),
            AccountMeta::new(enc_account_b, false),
        ],
    );
    let blockhash = rpc.get_latest_blockhash()?;
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(&[&payer], blockhash);
    let transfer_sig = rpc.send_and_confirm_transaction(&tx)?;
    println!("✅ Confidential Transfer Tx: https://solscan.io/tx/{}?cluster=devnet", transfer_sig);

    // ----------------------------------------------------
    // PHASE 4: Unshield Funds (Withdraw SOL)
    // ----------------------------------------------------
    let unshield_amount: u64 = 200_000; // 0.0002 SOL
    println!("\n➡️  Unshielding {} lamports (0.0002 SOL) back to Sender wallet...", unshield_amount);
    let mut disc = get_discriminator("unshield_funds").to_vec();
    disc.extend_from_slice(&unshield_amount.to_le_bytes());
    disc.push(vault_bump);

    let ix = Instruction::new_with_bytes(
        program_id,
        &disc,
        vec![
            AccountMeta::new_readonly(payer.pubkey(), true),
            AccountMeta::new(registry_pda, false),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(payer.pubkey(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    let blockhash = rpc.get_latest_blockhash()?;
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(&[&payer], blockhash);
    let unshield_sig = rpc.send_and_confirm_transaction(&tx)?;
    println!("✅ Unshield Tx: https://solscan.io/tx/{}?cluster=devnet", unshield_sig);

    println!("\n======================================================");
    println!("🎉 FHE DEVNET CYCLE COMPLETE!");
    println!("======================================================");
    println!("Program  : {}", program_id);
    println!("Sender   : {}", payer.pubkey());
    println!("Receiver : {}", receiver_keypair.pubkey());
    println!("\nTransactions:");
    println!("  Shield   : https://solscan.io/tx/{}?cluster=devnet", shield_sig);
    println!("  Transfer : https://solscan.io/tx/{}?cluster=devnet", transfer_sig);
    println!("  Unshield : https://solscan.io/tx/{}?cluster=devnet", unshield_sig);
    println!("\nFHE Ciphertext Hashes committed on-chain:");
    println!("  Sender Commitment  : hex:{}", hex::encode(sender_hash));
    println!("  Receiver Commitment: hex:{}", hex::encode(receiver_hash));

    Ok(())
}
