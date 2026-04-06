use std::fmt;

/// Global error type for all FHESTATE library operations.
///
/// Each variant maps to a specific failure domain so callers can handle
/// errors precisely without matching on string messages.
#[derive(Debug)]
pub enum FheError {
    // ── Key Management ────────────────────────────────────────────────────
    /// FHE key generation failed (e.g. out of memory, bad config).
    KeyGenFailed(String),
    /// A key file could not be read or deserialised.
    KeyLoadFailed(String),
    /// The expected key file does not exist on disk.
    KeyNotFound(String),
    /// The key bytes are not in the expected bincode format.
    InvalidKeyFormat,
    /// Homomorphic operations were attempted before `set_server_key` was called.
    ServerKeyNotActive,

    // ── I/O & Serialisation ───────────────────────────────────────────────
    /// Filesystem error (wrapped std::io::Error).
    Io(std::io::Error),
    /// Bincode serialisation or deserialisation error.
    Serialization(bincode::Error),

    // ── Solana / RPC ──────────────────────────────────────────────────────
    /// Solana JSON-RPC call failed.
    RpcError(String),
    /// A signed transaction was rejected or failed to confirm.
    TransactionFailed(String),
    /// The on-chain program account was not found at the expected address.
    ProgramNotFound(String),

    // ── FHE Computation ───────────────────────────────────────────────────
    /// An `op` code was not recognised by `FheMath::execute_op`.
    /// The inner `u8` is the unrecognised operation code.
    InvalidOperation(u8),
    /// A homomorphic computation produced an unexpected result or panicked.
    ComputationFailed(String),
    /// A task did not complete within the configured timeout window.
    TaskTimeout(u64),

    // ── Cache ─────────────────────────────────────────────────────────────
    /// The requested ciphertext URI was not found in the local cache.
    CacheMiss(String),
}

impl fmt::Display for FheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FheError::KeyGenFailed(e) => write!(f, "[fhestate] Key generation failed: {}", e),
            FheError::KeyLoadFailed(e) => write!(f, "[fhestate] Failed to load key: {}", e),
            FheError::KeyNotFound(p) => write!(f, "[fhestate] Key file not found: {}", p),
            FheError::InvalidKeyFormat => write!(
                f,
                "[fhestate] Invalid key format (expected bincode-serialised TFHE-rs key)"
            ),
            FheError::ServerKeyNotActive => write!(
                f,
                "[fhestate] Server key not activated — call activate_server_key() first"
            ),
            FheError::Io(e) => write!(f, "[fhestate] IO error: {}", e),
            FheError::Serialization(e) => write!(f, "[fhestate] Serialization error: {}", e),
            FheError::RpcError(e) => write!(f, "[fhestate] Solana RPC error: {}", e),
            FheError::TransactionFailed(e) => write!(f, "[fhestate] Transaction failed: {}", e),
            FheError::ProgramNotFound(p) => write!(f, "[fhestate] Program not found: {}", p),
            FheError::InvalidOperation(o) => write!(
                f,
                "[fhestate] Invalid operation code: {} (see constants::ops)",
                o
            ),
            FheError::ComputationFailed(e) => write!(f, "[fhestate] FHE computation failed: {}", e),
            FheError::TaskTimeout(t) => write!(f, "[fhestate] Task timed out after {} seconds", t),
            FheError::CacheMiss(u) => write!(f, "[fhestate] Cache miss for URI: {}", u),
        }
    }
}

impl std::error::Error for FheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FheError::Io(e) => Some(e),
            FheError::Serialization(e) => Some(e),
            _ => None,
        }
    }
}

// ── From implementations ──────────────────────────────────────────────────────

impl From<std::io::Error> for FheError {
    fn from(err: std::io::Error) -> Self {
        FheError::Io(err)
    }
}

impl From<bincode::Error> for FheError {
    fn from(err: bincode::Error) -> Self {
        FheError::Serialization(err)
    }
}

/// Convenient alias — all FHESTATE functions return this.
pub type FheResult<T> = Result<T, FheError>;

// ── Error code helpers ────────────────────────────────────────────────────────

impl FheError {
    /// Returns true if this error is likely transient and worth retrying
    /// (e.g. RPC failure, cache miss on a URI that may not have been uploaded yet).
    pub fn is_retryable(&self) -> bool {
        matches!(self, FheError::RpcError(_) | FheError::CacheMiss(_))
    }

    /// Returns true if this error is a key-management issue requiring user action.
    pub fn is_key_error(&self) -> bool {
        matches!(
            self,
            FheError::KeyGenFailed(_)
                | FheError::KeyLoadFailed(_)
                | FheError::KeyNotFound(_)
                | FheError::InvalidKeyFormat
                | FheError::ServerKeyNotActive
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_contains_prefix() {
        let e = FheError::InvalidOperation(99);
        assert!(e.to_string().contains("[fhestate]"));
        assert!(e.to_string().contains("99"));
    }

    #[test]
    fn test_is_retryable() {
        assert!(FheError::RpcError("timeout".into()).is_retryable());
        assert!(FheError::CacheMiss("local://abc".into()).is_retryable());
        assert!(!FheError::InvalidKeyFormat.is_retryable());
    }

    #[test]
    fn test_is_key_error() {
        assert!(FheError::ServerKeyNotActive.is_key_error());
        assert!(FheError::KeyNotFound("path".into()).is_key_error());
        assert!(!FheError::TaskTimeout(60).is_key_error());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let fhe_err = FheError::from(io_err);
        assert!(matches!(fhe_err, FheError::Io(_)));
    }
}
