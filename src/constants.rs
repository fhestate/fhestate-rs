/// Security level in bits (TFHE-rs default).
pub const SECURITY_LEVEL: u32 = 128;

/// Default Solana RPC endpoint (Devnet).
pub const DEFAULT_RPC: &str = "https://api.devnet.solana.com";

/// Default directory for FHE keys.
pub const KEY_DIR: &str = "fhe_keys";

/// Default directory for ciphertext cache.
pub const CACHE_DIR: &str = ".fhe_cache";

/// Maximum task execution time in seconds.
pub const TASK_TIMEOUT_SECS: u64 = 600;

/// Chain polling interval in seconds.
pub const POLL_INTERVAL_SECS: u64 = 2;

/// Estimated ciphertext size for FheUint8 (bytes).
pub const CT_U8_SIZE: usize = 8_192;

/// Estimated ciphertext size for FheUint32 (bytes).
pub const CT_U32_SIZE: usize = 32_768;

/// Operation codes for on-chain task dispatch.
pub mod ops {
    pub const ADD: u8 = 0;
    pub const SUB: u8 = 1;
    pub const MUL: u8 = 2;
    pub const CMP: u8 = 3;
    pub const AND: u8 = 4;
    pub const OR: u8 = 5;
    pub const XOR: u8 = 6;
}
