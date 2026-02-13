//! Off-chain state transition engine.
//!
//! Responsible for:
//! 1. Loading the current encrypted state from [`LocalCache`] (or bootstrapping from zero).
//! 2. Applying an FHE operation on the input ciphertext.
//! 3. Saving the new encrypted state back to [`LocalCache`].
//! 4. Returning the new cache URI and a SHA256 proof hash.

use crate::cache::LocalCache;
use crate::errors::{FheError, FheResult};
use crate::math::FheMath;
use sha2::{Digest, Sha256};

/// Off-chain FHE state transition engine.
pub struct StateTransition;

impl StateTransition {
    /// Apply an FHE operation to the current encrypted state.
    ///
    /// Steps:
    /// 1. Load (or bootstrap) the current `FheUint32` ciphertext from cache.
    /// 2. Deserialise the input ciphertext provided by the submitter.
    /// 3. Apply `op` homomorphically.
    /// 4. Serialise and store the new state ciphertext.
    /// 5. Return `(new_cache_uri, sha256_of_new_state_bytes)`.
    ///
    /// # Arguments
    /// * `cache`       - Local ciphertext cache.
    /// * `state_uri`   - Current state URI, or `None` for a fresh account (bootstraps from input).
    /// * `input_bytes` - Serialised `FheUint32` ciphertext from the submitter.
    /// * `op`          - Operation code (see `crate::constants::ops`).
    pub fn apply(
        cache: &LocalCache,
        state_uri: Option<&str>,
        input_bytes: &[u8],
        op: u8,
    ) -> FheResult<(String, [u8; 32])> {
        if input_bytes.is_empty() {
            return Err(FheError::ComputationFailed(
                "input_bytes must not be empty".to_string(),
            ));
        }

        // Deserialise the submitter's input ciphertext.
        let input_ct = FheMath::deserialize_u32(input_bytes)?;

        // Compute the new state.
        let new_state_ct = match state_uri {
            None => {
                // No prior state: treat the input itself as the new state.
                log::info!("StateTransition: fresh account — using input as initial state");
                input_ct
            }
            Some(uri) => {
                // Load the old state ciphertext.
                let old_bytes = cache.load(uri)?;
                let old_ct = FheMath::deserialize_u32(&old_bytes)?;

                // Apply the requested FHE op.
                FheMath::execute_op(op, &old_ct, &input_ct).ok_or_else(|| {
                    FheError::InvalidOperation(op)
                })?
            }
        };

        // Serialise the new state and persist it.
        let new_state_bytes = FheMath::serialize_u32(&new_state_ct)?;
        let new_uri = cache.store(&new_state_bytes)?;

        // Compute SHA256 proof hash of the new state bytes.
        let mut hasher = Sha256::new();
        hasher.update(&new_state_bytes);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&hasher.finalize());

        log::info!("StateTransition: op={} -> new_uri={}", op, new_uri);
        Ok((new_uri, hash))
    }
}
