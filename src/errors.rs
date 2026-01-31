use std::fmt;

/// Global error type for FHEstate SDK operations.
#[derive(Debug)]
pub enum FheError {
    KeyGenFailed(String),
    KeyLoadFailed(String),
    KeyNotFound(String),
    Io(std::io::Error),
    Serialization(bincode::Error),
    InvalidKeyFormat,
    ServerKeyNotActive,
    RpcError(String),
    TransactionFailed(String),
    TaskTimeout(u64),
    CacheMiss(String),
    ComputationFailed(String),
    InvalidOperation(u8),
    ProgramNotFound(String),
}

impl fmt::Display for FheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FheError::KeyGenFailed(e) => write!(f, "Key generation failed: {}", e),
            FheError::KeyLoadFailed(e) => write!(f, "Failed to load key: {}", e),
            FheError::KeyNotFound(p) => write!(f, "Key file not found: {}", p),
            FheError::Io(e) => write!(f, "IO error: {}", e),
            FheError::Serialization(e) => write!(f, "Serialization error: {}", e),
            FheError::InvalidKeyFormat => write!(f, "Invalid key format"),
            FheError::ServerKeyNotActive => write!(f, "Server key not activated"),
            FheError::RpcError(e) => write!(f, "RPC error: {}", e),
            FheError::TransactionFailed(e) => write!(f, "Transaction failed: {}", e),
            FheError::TaskTimeout(t) => write!(f, "Task timeout after {} seconds", t),
            FheError::CacheMiss(u) => write!(f, "Cache miss for URI: {}", u),
            FheError::ComputationFailed(e) => write!(f, "Computation failed: {}", e),
            FheError::InvalidOperation(o) => write!(f, "Invalid operation code: {}", o),
            FheError::ProgramNotFound(p) => write!(f, "Program not found: {}", p),
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

pub type FheResult<T> = Result<T, FheError>;
