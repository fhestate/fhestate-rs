//! Off-chain state transition engine.
//!
//! Responsible for:
//! 1. Loading the current encrypted state from [`LocalCache`] (or bootstrapping from zero).
//! 2. Applying an FHE operation on the input ciphertext.
//! 3. Saving the new encrypted state back to [`LocalCache`].
//! 4. Returning the new cache URI and a SHA256 proof hash.

use crate::cache::LocalCache;
use crate::errors::{FheError, FheResult};
use sha2::{Digest, Sha256};

/// Off-chain FHE state transition engine.
///
/// Encapsulates the load → compute → save cycle so the fhe-node service
/// can call a single method without knowing the caching internals.
pub struct StateTransition;

impl StateTransition {
    /// Apply an FHE operation to the current encrypted state.
    ///
    /// # Arguments
    /// * `cache`     - Local ciphertext cache.
    /// * `state_uri` - URI of the current encrypted state, or `None` for a fresh account.
    /// * `input_bytes` - Serialised `FheUint32` ciphertext provided by the submitter.
    /// * `op`        - Operation code (from `constants::ops`).
    ///
    /// # Returns
    /// `(new_state_uri, sha256_of_new_state_bytes)` on success.
    pub fn apply(
        cache: &LocalCache,
        state_uri: Option<&str>,
        input_bytes: &[u8],
        op: u8,
    ) -> FheResult<(String, [u8; 32])> {
        // Validate input is non-empty before touching FHE context.
        if input_bytes.is_empty() {
            return Err(FheError::ComputationFailed(
                "input_bytes must not be empty".to_string(),
            ));
        }

        // Load or bootstrap the current state ciphertext bytes.
        let _state_bytes = match state_uri {
            Some(uri) => cache.load(uri)?,
            None => {
                log::info!("StateTransition: no prior state — bootstrapping from zero");
                Vec::new() // placeholder; Week 2 wires real FheUint32::encrypt(0)
            }
        };

        // Compute new state bytes (full FHE computation wired in Week 2 service update).
        // For now, store the input as the new state so the URI / hash pipeline is testable.
        let new_state_bytes = input_bytes.to_vec();

        // Persist new state.
        let new_uri = cache.store(&new_state_bytes)?;

        // Compute SHA256 proof hash of the new state.
        let mut hasher = Sha256::new();
        hasher.update(&new_state_bytes);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&hasher.finalize());

        log::info!("StateTransition: op={} -> new_uri={}", op, new_uri);
        Ok((new_uri, hash))
    }
}
