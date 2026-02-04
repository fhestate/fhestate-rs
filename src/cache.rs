use crate::constants::CACHE_DIR;
use crate::errors::{FheError, FheResult};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

/// Local file-based cache for ciphertexts.
/// Uses content-addressed storage (SHA256 hash of content as filename).
/// This replaces Arweave for local development and testing.
pub struct LocalCache {
    dir: String,
}

impl LocalCache {
    /// Create cache with specified directory.
    pub fn new(dir: &str) -> Self {
        if let Err(e) = fs::create_dir_all(dir) {
            log::warn!("Failed to create cache directory: {}", e);
        }
        Self {
            dir: dir.to_string(),
        }
    }

    /// Create cache with default directory.
    pub fn default() -> Self {
        Self::new(CACHE_DIR)
    }

    /// Store bytes and return content-addressed URI.
    /// Format: local://<hash_hex>
    pub fn store(&self, data: &[u8]) -> FheResult<String> {
        let hash = self.hash_bytes(data);
        // Use full 32-byte hash — was &hash[0..16] which doubled collision risk
        let hash_hex = hex::encode(&hash);
        let path = format!("{}/{}.bin", self.dir, hash_hex);

        let mut file = File::create(&path)?;
        file.write_all(data)?;

        Ok(format!("local://{}", hash_hex))
    }

    /// Load bytes from URI.
    pub fn load(&self, uri: &str) -> FheResult<Vec<u8>> {
        let hash_hex = uri.trim_start_matches("local://");
        let path = format!("{}/{}.bin", self.dir, hash_hex);

        if !Path::new(&path).exists() {
            return Err(FheError::CacheMiss(uri.to_string()));
        }

        let mut file = File::open(&path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(data)
    }

    /// Check if URI exists in cache.
    pub fn exists(&self, uri: &str) -> bool {
        let hash_hex = uri.trim_start_matches("local://");
        let path = format!("{}/{}.bin", self.dir, hash_hex);
        Path::new(&path).exists()
    }

    /// Delete item by URI.
    pub fn delete(&self, uri: &str) -> FheResult<()> {
        let hash_hex = uri.trim_start_matches("local://");
        let path = format!("{}/{}.bin", self.dir, hash_hex);
        fs::remove_file(path)?;
        Ok(())
    }

    /// Clear all cached items.
    pub fn clear(&self) -> FheResult<()> {
        fs::remove_dir_all(&self.dir)?;
        fs::create_dir_all(&self.dir)?;
        Ok(())
    }

    /// Get total cache size in bytes.
    pub fn size(&self) -> FheResult<u64> {
        let mut total = 0u64;
        if let Ok(entries) = fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            }
        }
        Ok(total)
    }

    /// List all cached URIs.
    pub fn list(&self) -> FheResult<Vec<String>> {
        let mut uris = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".bin") {
                        let hash = name.trim_end_matches(".bin");
                        uris.push(format!("local://{}", hash));
                    }
                }
            }
        }
        Ok(uris)
    }

    fn hash_bytes(&self, data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let mut out = [0u8; 32];
        out.copy_from_slice(&hasher.finalize());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> LocalCache {
        LocalCache::new(&format!(".fhe_test_cache_{}", std::process::id()))
    }

    #[test]
    fn test_store_load_roundtrip() {
        let c = tmp();
        let data = b"fhestate ciphertext roundtrip";
        let uri = c.store(data).unwrap();
        assert!(uri.starts_with("local://"));
        assert_eq!(c.load(&uri).unwrap(), data);
        let _ = c.clear();
    }

    #[test]
    fn test_cache_miss_returns_err() {
        let c = tmp();
        assert!(c.load("local://nonexistent_hash_xyz").is_err());
    }

    #[test]
    fn test_uri_uses_full_32_byte_hash() {
        let c = tmp();
        let uri = c.store(b"test").unwrap();
        // "local://" = 8 chars, SHA256 hex = 64 chars → total 72
        assert_eq!(uri.len(), 72, "URI must encode full 32-byte SHA256 hash");
        let _ = c.clear();
    }
}
