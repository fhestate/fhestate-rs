use crate::constants::KEY_DIR;
use crate::errors::{FheError, FheResult};
use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use tfhe::{generate_keys, set_server_key, ClientKey, ConfigBuilder, ServerKey};

/// Manages FHE keypair lifecycle (generation, storage, loading).
pub struct KeyManager {
    pub client_key: ClientKey,
    pub server_key: ServerKey,
}

impl KeyManager {
    /// Generate fresh FHE keypair with 128-bit security.
    /// This operation takes 30-60 seconds on typical hardware.
    pub fn generate() -> FheResult<Self> {
        println!("   [1/3] Configuring FHE parameters...");
        let config = ConfigBuilder::default().build();

        println!("   [2/3] Generating keys (this involves heavy cryptography)...");
        let (client_key, server_key) = generate_keys(config);

        println!("   [3/3] Keys generated successfully.");
        Ok(Self {
            client_key,
            server_key,
        })
    }

    /// Activate server key for homomorphic computations on current thread.
    pub fn activate(&self) {
        set_server_key(self.server_key.clone());
    }

    /// Save keypair to specified directory.
    pub fn save(&self, dir: &str) -> FheResult<()> {
        fs::create_dir_all(dir)?;

        println!("   [Save 1/2] Saving Client Key...");
        let client_path = format!("{}/client_key.bin", dir);
        let mut client_file = BufWriter::new(File::create(&client_path)?);
        bincode::serialize_into(&mut client_file, &self.client_key)?;
        client_file.flush()?;
        println!("              Saved: {}", client_path);

        println!("   [Save 2/2] Saving Server Key (this is large, please wait)...");
        let server_path = format!("{}/server_key.bin", dir);
        let mut server_file = BufWriter::new(File::create(&server_path)?);
        bincode::serialize_into(&mut server_file, &self.server_key)?;
        server_file.flush()?;
        println!("              Saved: {}", server_path);

        Ok(())
    }

    /// Load keypair from specified directory.
    pub fn load(dir: &str) -> FheResult<Self> {
        let client_key = load_client_key(&format!("{}/client_key.bin", dir))?;
        let server_key = load_server_key(&format!("{}/server_key.bin", dir))?;
        Ok(Self {
            client_key,
            server_key,
        })
    }

    /// Load from default directory.
    pub fn load_default() -> FheResult<Self> {
        Self::load(KEY_DIR)
    }
}

/// Load client key from file path.
pub fn load_client_key(path: &str) -> FheResult<ClientKey> {
    if !Path::new(path).exists() {
        return Err(FheError::KeyNotFound(path.to_string()));
    }
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let key: ClientKey = bincode::deserialize(&bytes)?;
    Ok(key)
}

/// Load server key from file path.
pub fn load_server_key(path: &str) -> FheResult<ServerKey> {
    if !Path::new(path).exists() {
        return Err(FheError::KeyNotFound(path.to_string()));
    }
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let key: ServerKey = bincode::deserialize(&bytes)?;
    Ok(key)
}

/// Activate server key globally for FHE operations.
pub fn activate_server_key(key: &ServerKey) {
    set_server_key(key.clone());
}

/// Check if keys exist in specified directory.
pub fn keys_exist(dir: &str) -> bool {
    Path::new(&format!("{}/client_key.bin", dir)).exists()
        && Path::new(&format!("{}/server_key.bin", dir)).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keys_exist_false_for_missing_dir() {
        assert!(!keys_exist("/totally_nonexistent_fhe_dir_9x7z"),
            "keys_exist must return false when directory does not exist");
    }

    #[test]
    fn test_keys_exist_false_when_dir_is_empty() {
        let dir = format!(".fhe_test_keys_{}", std::process::id());
        std::fs::create_dir_all(&dir).unwrap();
        assert!(!keys_exist(&dir), "keys_exist must return false when key files are absent");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
