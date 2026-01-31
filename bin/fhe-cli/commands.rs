use fhestate_sdk::keys::keys_exist;
use fhestate_sdk::{FheMath, KeyManager};
use log::{error, info};
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use tfhe::FheUint8;
use tfhe::prelude::*;

pub fn keygen(out_dir: &str) -> Result<(), Box<dyn Error>> {
    info!("FHEstate Key Generation");
    info!("   Process started (30-60s)...");

    let km = KeyManager::generate().map_err(|e| format!("Key generation failed: {}", e))?;

    km.save(out_dir)
        .map_err(|e| format!("Failed to save keys: {}", e))?;

    info!("   Keys saved successfully in {}", out_dir);
    Ok(())
}

pub fn wallet(output: &str) -> Result<(), Box<dyn Error>> {
    info!("Generating Solana Wallet");

    let keypair = Keypair::new();
    let keypair_bytes: Vec<u8> = keypair.to_bytes().to_vec();

    let mut file = File::create(output)?;
    serde_json::to_writer(&mut file, &keypair_bytes)?;

    info!("   Wallet saved to {}", output);
    info!("   Public Key: {}", keypair.pubkey());

    Ok(())
}

pub fn proof(_rpc_url: &str) -> Result<(), Box<dyn Error>> {
    info!("FHEstate Proof Generation");
    info!("   Target: 'Solana Privacy Ops'");

    if !keys_exist("fhe_keys") {
        return Err("Keys not found. Run 'keygen' first.".into());
    }

    info!("   Loading keys...");
    let km = KeyManager::load("fhe_keys")?;
    km.activate();
    info!("   Context activated.");

    let sentence = "Solana Privacy Ops";
    print!("   Encrypting: ");
    let mut ciphertexts: Vec<FheUint8> = Vec::new();
    for byte in sentence.as_bytes() {
        let ct = FheMath::encrypt_u8(*byte, &km.client_key);
        ciphertexts.push(ct);
        print!(".");
        std::io::stdout().flush()?;
    }
    println!(" [Done]");

    info!("   Performing Homomorphic Shift (+1)...");
    let one = FheMath::encrypt_u8(1u8, &km.client_key);
    let mut shifted: Vec<FheUint8> = Vec::new();
    for ct in &ciphertexts {
        let result = ct + &one;
        shifted.push(result);
    }

    info!("   Decrypting result...");
    let mut decrypted = String::new();
    for ct in &shifted {
        let val = FheMath::decrypt_u8(ct, &km.client_key);
        decrypted.push(val as char);
    }

    info!("   Original:  {}", sentence);
    info!("   Decrypted: {}", decrypted);

    let unshifted: String = decrypted.chars().map(|c| (c as u8 - 1) as char).collect();
    if unshifted == sentence {
        info!("   Status: VERIFIED SUCCESS");
    } else {
        error!("   Status: VERIFICATION FAILED");
    }

    Ok(())
}

pub fn submit_task(
    rpc_url: &str,
    program_id: &str,
    wallet_path: &str,
    op: u8,
) -> Result<(), Box<dyn Error>> {
    info!("Submitting FHE Task to Solana");

    // 1. Load Wallet
    let wallet_file = File::open(wallet_path)?;
    let wallet_bytes: Vec<u8> = serde_json::from_reader(wallet_file)?;
    let payer = Keypair::from_bytes(&wallet_bytes)?;
    info!("   Submitter: {}", payer.pubkey());

    // 2. Prepare Data
    let prog_id = Pubkey::from_str(program_id)?;
    let rpc = RpcClient::new(rpc_url.to_string());

    // Load FHE client key for encryption
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::path::Path;
    use tfhe::{ClientKey, FheUint8};

    let client_key_path = Path::new("fhe_keys/client_key.bin");
    if !client_key_path.exists() {
        return Err("Client key not found. Run keygen first.".into());
    }

    let client_key_bytes = fs::read(client_key_path)?;
    let client_key: ClientKey = bincode::deserialize(&client_key_bytes)?;

    // Encrypt operation input
    let input_value = (op as u8).wrapping_mul(7);
    let encrypted_input = FheUint8::encrypt(input_value, &client_key);

    // Serialize and hash the encrypted input
    let ciphertext_bytes = bincode::serialize(&encrypted_input)?;
    let mut hasher = Sha256::new();
    hasher.update(&ciphertext_bytes);
    let input_hash: [u8; 32] = hasher.finalize().into();

    // Build task submission data with hash proof
    let task_data = format!(
        "FHE_TASK_SUBMISSION:OP={}:HASH={}",
        op,
        hex::encode(&input_hash[..16])
    );
    let data = task_data.as_bytes().to_vec();

    let ix =
        Instruction::new_with_bytes(prog_id, &data, vec![AccountMeta::new(payer.pubkey(), true)]);

    let recent_blockhash = rpc.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    info!("   Sending Transaction...");
    let signature = rpc.send_and_confirm_transaction(&tx)?;
    info!("   Success! Transaction Hash: {}", signature);

    Ok(())
}
