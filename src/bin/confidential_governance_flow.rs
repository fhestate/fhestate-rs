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

fn fetch_instruction_discriminator(name: &str) -> [u8; 8] {
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
    println!("🛡️  FHESTATE: CONFIDENTIAL GOVERNANCE INTEGRATION TEST");
    println!("======================================================");

    // 1. Establish RPC connection to Devnet
    let rpc_url = "https://api.devnet.solana.com".to_string();
    println!("Connecting to Solana Devnet RPC: {}", rpc_url);
    let rpc = RpcClient::new(rpc_url);

    // 2. Load admin authority wallet
    let wallet_path = "deploy-wallet.json";
    println!("Loading admin/authority keypair from '{}'...", wallet_path);
    let file = File::open(wallet_path)?;
    let bytes: Vec<u8> = serde_json::from_reader(file)?;
    let admin = Keypair::from_bytes(&bytes)?;
    println!("Admin Public Address: {}", admin.pubkey());

    // 3. Define target program IDs and PDAs
    let program_id = Pubkey::from_str("FuQzZCwPSRSVLT9gCgcft43a4RkapBJmSTC6CmdomeVQ")?;
    let (registry_pda, _) = Pubkey::find_program_address(&[b"vault_registry"], &program_id);
    println!("Vault Registry PDA: {}", registry_pda);
    
    // Check and initialize Vault Registry PDA if not already initialized
    if rpc.get_account(&registry_pda).is_err() {
        println!("Vault Registry PDA not initialized. Initializing now...");
        let mut init_ix_data = fetch_instruction_discriminator("initialize_vault").to_vec();
        // Pass admin.pubkey() as the attestation authority
        init_ix_data.extend_from_slice(&admin.pubkey().to_bytes());

        let init_instruction = Instruction::new_with_bytes(
            program_id,
            &init_ix_data,
            vec![
                AccountMeta::new(registry_pda, false),
                AccountMeta::new(admin.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let blockhash = rpc.get_latest_blockhash()?;
        let mut tx_init = Transaction::new_with_payer(&[init_instruction], Some(&admin.pubkey()));
        tx_init.sign(&[&admin], blockhash);
        let init_sig = rpc.send_and_confirm_transaction(&tx_init)?;
        println!("✅ Vault Registry PDA Initialized. Transaction Hash: {}", init_sig);
    } else {
        println!("Vault Registry PDA is already initialized.");
    }

    // 4. Update the on-chain homomorphic spending limit (Pillar 05)
    println!("\n[1/3] Updating Confidential spending limit hash on-chain...");
    let client_key = load_client_key("fhe_keys/client_key.bin")?;
    let server_key = load_server_key("fhe_keys/server_key.bin")?;
    activate_server_key(&server_key);
    
    // Encrypt the target daily spending limit (e.g. 5,000,000 lamports) homomorphically
    let target_limit: u32 = 5_000_000;
    let limit_ct = FheUint32::encrypt(target_limit, &client_key);
    let limit_ct_bytes = bincode::serialize(&limit_ct)?;
    
    let mut hasher = Sha256::new();
    hasher.update(&limit_ct_bytes);
    let mut limit_hash = [0u8; 32];
    limit_hash.copy_from_slice(&hasher.finalize());

    let mut limit_ix_data = fetch_instruction_discriminator("update_treasury_limit").to_vec();
    limit_ix_data.extend_from_slice(&limit_hash);

    let limit_instruction = Instruction::new_with_bytes(
        program_id,
        &limit_ix_data,
        vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(registry_pda, false),
        ],
    );

    let blockhash = rpc.get_latest_blockhash()?;
    let mut tx_limit = Transaction::new_with_payer(&[limit_instruction], Some(&admin.pubkey()));
    tx_limit.sign(&[&admin], blockhash);
    let limit_sig = rpc.send_and_confirm_transaction(&tx_limit)?;
    println!("✅ Spending Limit Hash Updated. Transaction Hash: {}", limit_sig);

    // 5. Initialize a Dark DAO Proposal on-chain (Pillar 06)
    let proposal_id: u64 = 301;
    let (proposal_pda, _) = Pubkey::find_program_address(
        &[b"proposal", &proposal_id.to_le_bytes()],
        &program_id
    );
    println!("\n[2/3] Initializing Confidential DAO Proposal PDA: {}...", proposal_pda);

    if rpc.get_account(&proposal_pda).is_err() {
        let mut prop_ix_data = fetch_instruction_discriminator("initialize_proposal").to_vec();
        prop_ix_data.extend_from_slice(&proposal_id.to_le_bytes());

        let prop_instruction = Instruction::new_with_bytes(
            program_id,
            &prop_ix_data,
            vec![
                AccountMeta::new(proposal_pda, false),
                AccountMeta::new(admin.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let blockhash = rpc.get_latest_blockhash()?;
        let mut tx_prop = Transaction::new_with_payer(&[prop_instruction], Some(&admin.pubkey()));
        tx_prop.sign(&[&admin], blockhash);
        let prop_sig = rpc.send_and_confirm_transaction(&tx_prop)?;
        println!("✅ Proposal PDA Initialized. Transaction Hash: {}", prop_sig);
    } else {
        println!("Proposal PDA is already initialized on-chain.");
    }

    // 6. Local FHE verification of DAO vote aggregation (Tree-Sum)
    println!("\n[3/3] Simulating local homomorphic ballot accumulation...");
    
    // Simulate 5 individual encrypted YES votes and 2 encrypted NO votes
    let encrypted_yes_votes: Vec<FheUint32> = (0..5).map(|_| FheUint32::encrypt(1u32, &client_key)).collect();
    let encrypted_no_votes: Vec<FheUint32> = (0..2).map(|_| FheUint32::encrypt(1u32, &client_key)).collect();

    // Accumulate the votes homomorphically
    let mut accumulated_yes = FheUint32::encrypt(0u32, &client_key);
    for vote in encrypted_yes_votes {
        accumulated_yes = &accumulated_yes + &vote;
    }

    let mut accumulated_no = FheUint32::encrypt(0u32, &client_key);
    for vote in encrypted_no_votes {
        accumulated_no = &accumulated_no + &vote;
    }

    // Decrypt in secure local environment (simulating TEE coordinator)
    let decrypted_yes: u32 = accumulated_yes.decrypt(&client_key);
    let decrypted_no: u32 = accumulated_no.decrypt(&client_key);

    println!("📊 Tally verification results:");
    println!("   - YES vote count (decrypted): {}", decrypted_yes);
    println!("   - NO vote count (decrypted) : {}", decrypted_no);
    assert_eq!(decrypted_yes, 5);
    assert_eq!(decrypted_no, 2);

    println!("\n======================================================");
    println!("🎉 CONFIDENTIAL GOVERNANCE INTEGRATION TEST VERIFIED!");
    println!("======================================================");

    Ok(())
}
