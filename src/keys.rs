use crate::constants::KEY_DIR;
use crate::errors::{FheError, FheResult};
use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use tfhe::{generate_keys, set_server_key, ClientKey, ConfigBuilder, ServerKey};
use tracing::{info, warn};

/// Manages FHE keypair lifecycle (generation, storage, loading).
pub struct KeyManager {
    pub client_key: ClientKey,
    pub server_key: ServerKey,
}

impl KeyManager {
    /// Generate a fresh FHE keypair with 128-bit security.
    ///
    /// ⚠️  This operation takes **30–90 seconds** on typical hardware due to
    /// the underlying TFHE-rs lattice parameter setup. This is expected behaviour.
    pub fn generate() -> FheResult<Self> {
        info!("configuring FHE parameters (128-bit security)");
        let config = ConfigBuilder::default().build();

        info!("generating keypair — this may take 30–90 seconds");
        let (client_key, server_key) = generate_keys(config);

        info!("keypair generation complete");
        Ok(Self {
            client_key,
            server_key,
        })
    }

    /// Activate the server key for homomorphic computations on the current thread.
    /// Must be called before any FHE operation.
    pub fn activate(&self) {
        set_server_key(self.server_key.clone());
        info!("server key activated on current thread");
    }

    /// Save keypair to `dir`. Creates the directory if it does not exist.
    pub fn save(&self, dir: &str) -> FheResult<()> {
        fs::create_dir_all(dir)?;

        let client_path = format!("{}/client_key.bin", dir);
        info!(path = %client_path, "saving client key");
        let mut client_file = BufWriter::new(File::create(&client_path)?);
        bincode::serialize_into(&mut client_file, &self.client_key)?;
        client_file.flush()?;

        let server_path = format!("{}/server_key.bin", dir);
        info!(path = %server_path, "saving server key (large file — please wait)");
        let mut server_file = BufWriter::new(File::create(&server_path)?);
        bincode::serialize_into(&mut server_file, &self.server_key)?;
        server_file.flush()?;

        info!(dir = %dir, "keypair saved");
        Ok(())
    }

    /// Load keypair from `dir`.
    pub fn load(dir: &str) -> FheResult<Self> {
        info!(dir = %dir, "loading keypair");
        let client_key = load_client_key(&format!("{}/client_key.bin", dir))?;
        let server_key = load_server_key(&format!("{}/server_key.bin", dir))?;
        info!("keypair loaded successfully");
        Ok(Self {
            client_key,
            server_key,
        })
    }

    /// Load from the default key directory (`fhe_keys/`).
    pub fn load_default() -> FheResult<Self> {
        Self::load(KEY_DIR)
    }
}

/// Load a client key from a file path.
pub fn load_client_key(path: &str) -> FheResult<ClientKey> {
    if !Path::new(path).exists() {
        warn!(path = %path, "client key file not found");
        return Err(FheError::KeyNotFound(path.to_string()));
    }
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let key: ClientKey =
        bincode::deserialize(&bytes).map_err(|_| FheError::KeyLoadFailed(path.to_string()))?;
    Ok(key)
}

/// Load a server key from a file path.
pub fn load_server_key(path: &str) -> FheResult<ServerKey> {
    if !Path::new(path).exists() {
        warn!(path = %path, "server key file not found");
        return Err(FheError::KeyNotFound(path.to_string()));
    }
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let key: ServerKey =
        bincode::deserialize(&bytes).map_err(|_| FheError::KeyLoadFailed(path.to_string()))?;
    Ok(key)
}

/// Activate a server key globally for FHE operations on the current thread.
pub fn activate_server_key(key: &ServerKey) {
    set_server_key(key.clone());
    info!("server key activated");
}

/// Returns true if both `client_key.bin` and `server_key.bin` exist in `dir`.
pub fn keys_exist(dir: &str) -> bool {
    Path::new(&format!("{}/client_key.bin", dir)).exists()
        && Path::new(&format!("{}/server_key.bin", dir)).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keys_exist_false_for_missing_dir() {
        assert!(
            !keys_exist("/totally_nonexistent_fhe_dir_9x7z"),
            "keys_exist must return false when directory does not exist"
        );
    }

    #[test]
    fn test_keys_exist_false_when_dir_is_empty() {
        let dir = format!(".fhe_test_keys_{}", std::process::id());
        std::fs::create_dir_all(&dir).unwrap();
        assert!(
            !keys_exist(&dir),
            "keys_exist must return false when key files are absent"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_missing_client_key_returns_key_not_found() {
        let result = load_client_key("/nonexistent/client_key.bin");
        assert!(matches!(result, Err(FheError::KeyNotFound(_))));
    }

    #[test]
    fn test_load_missing_server_key_returns_key_not_found() {
        let result = load_server_key("/nonexistent/server_key.bin");
        assert!(matches!(result, Err(FheError::KeyNotFound(_))));
    }
}

