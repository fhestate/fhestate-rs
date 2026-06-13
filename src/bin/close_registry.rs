use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
    instruction::{AccountMeta, Instruction},
};
use solana_client::rpc_client::RpcClient;
use std::str::FromStr;
use std::fs::File;
use std::error::Error;
use sha2::{Digest, Sha256};

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
    println!("Closing Vault Registry PDA to allow resizing...");
    
    let rpc_url = "https://api.devnet.solana.com".to_string();
    let rpc = RpcClient::new(rpc_url);

    let wallet_path = "deploy-wallet.json";
    let file = File::open(wallet_path)?;
    let bytes: Vec<u8> = serde_json::from_reader(file)?;
    let admin = Keypair::from_bytes(&bytes)?;
    println!("Admin Address: {}", admin.pubkey());

    let program_id = Pubkey::from_str("D14VbLLPcqkkZ6p4M9UDs4xfNdtB1tQDUqi7ZTt89etC")?;
    let (registry_pda, _) = Pubkey::find_program_address(&[b"vault_registry"], &program_id);
    println!("Registry PDA: {}", registry_pda);

    if rpc.get_account(&registry_pda).is_ok() {
        println!("Registry exists. Sending close_registry transaction...");
        let disc = get_discriminator("close_registry").to_vec();
        let ix = Instruction::new_with_bytes(
            program_id,
            &disc,
            vec![
                AccountMeta::new(admin.pubkey(), true),
                AccountMeta::new(registry_pda, false),
            ],
        );
        let blockhash = rpc.get_latest_blockhash()?;
        let mut tx = Transaction::new_with_payer(&[ix], Some(&admin.pubkey()));
        tx.sign(&[&admin], blockhash);
        let sig = rpc.send_and_confirm_transaction(&tx)?;
        println!("Registry successfully closed! Signature: https://solscan.io/tx/{}?cluster=devnet", sig);
    } else {
        println!("Registry PDA does not exist or is already closed.");
    }

    Ok(())
}
