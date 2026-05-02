use fhestate_rs::KeyManager;
use sha2::{Digest, Sha256};
use std::path::Path;
use tfhe::prelude::*;

pub fn ensure_fhe_keys(key_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    if fhestate_rs::keys::keys_exist(key_dir) {
        return Ok(());
    }
    crate::output::warn("No FHE keys found — generating (may take 30–60 seconds)...");
    let km = KeyManager::generate().map_err(|e| format!("Key generation failed: {e}"))?;
    km.save(key_dir)
        .map_err(|e| format!("Failed to save keys: {e}"))?;
    crate::output::ok(&format!("Keys saved to '{key_dir}/'"));
    Ok(())
}

pub fn encrypt_u32(value: u32, key_dir: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    ensure_fhe_keys(key_dir)?;
    let client_key_path = Path::new(key_dir).join("client_key.bin");
    let client_key_bytes = std::fs::read(&client_key_path)?;
    let client_key: tfhe::ClientKey = bincode::deserialize(&client_key_bytes)?;
    let encrypted = tfhe::FheUint32::encrypt(value, &client_key);
    Ok(bincode::serialize(&encrypted)?)
}

pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
