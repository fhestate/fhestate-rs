use solana_sdk::signature::Keypair;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn load_keypair(wallet_path: &str) -> Result<Keypair, Box<dyn std::error::Error>> {
    if !Path::new(wallet_path).exists() {
        return Err(format!(
            "Wallet file not found: '{}'. Run: fhe-cli wallet new --out {}",
            wallet_path, wallet_path
        )
        .into());
    }
    let wallet_bytes: Vec<u8> = serde_json::from_reader(File::open(wallet_path)?)?;
    Ok(Keypair::from_bytes(&wallet_bytes)?)
}

pub fn save_keypair(wallet_path: &str, keypair: &Keypair) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = Path::new(wallet_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let json = serde_json::to_string(&keypair.to_bytes().to_vec())?;
    let mut f = File::create(wallet_path)?;
    f.write_all(json.as_bytes())?;
    Ok(())
}

pub fn generate_wallet(wallet_path: &str) -> Result<Keypair, Box<dyn std::error::Error>> {
    let kp = Keypair::new();
    save_keypair(wallet_path, &kp)?;
    Ok(kp)
}
