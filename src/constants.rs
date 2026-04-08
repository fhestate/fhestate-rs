/// FHESTATE protocol version — bump this on any breaking change to
/// the on-chain account layout or ciphertext serialisation format.
/// The TypeScript SDK and fhe-node must match this version.
pub const PROTOCOL_VERSION: u8 = 1;

/// Crate version (mirrors Cargo.toml).
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

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
/// Used for transaction size validation and buffer pre-allocation.
pub const CT_U32_SIZE: usize = 32_768;

/// Maximum URI length stored on-chain (must match Anchor #[max_len]).
pub const MAX_URI_LEN: usize = 128;

/// Maximum proposal description length on-chain (must match Anchor #[max_len]).
pub const MAX_DESC_LEN: usize = 128;

/// Operation codes for on-chain task dispatch.
/// These values are stored in the `Task.operation` field and drive the
/// FHE computation inside `StateTransition::apply`.
pub mod ops {
    // ── Arithmetic ────────────────────────────────────────────────────────
    pub const ADD: u8 = 0;
    pub const SUB: u8 = 1;
    pub const MUL: u8 = 2;
    pub const CMP: u8 = 3; // Legacy: encrypted lt comparison, use LT instead

    // ── Bitwise ───────────────────────────────────────────────────────────
    pub const AND: u8 = 4;
    pub const OR: u8 = 5;
    pub const XOR: u8 = 6;

    // ── Encrypted Comparisons ─────────────────────────────────────────────
    // Returns FheUint32 encoding 1 (true) or 0 (false).
    pub const EQ: u8 = 10; // a == b
    pub const NE: u8 = 11; // a != b
    pub const GT: u8 = 12; // a >  b
    pub const LT: u8 = 13; // a <  b
    pub const GE: u8 = 14; // a >= b
    pub const LE: u8 = 15; // a <= b
    pub const MAX: u8 = 16; // max(a, b)
    pub const MIN: u8 = 17; // min(a, b)

    // ── Logical Primitives ────────────────────────────────────────────────
    pub const NOT: u8 = 20; // !a  (expects encrypted bool: 0 or 1)

    // ── Voting Operations ─────────────────────────────────────────────────
    pub const VOTE_TALLY: u8 = 30; // Accumulate encrypted ballots
    pub const CHECK_WINNER: u8 = 31; // Determine encrypted winner index
}

/// On-chain error codes mirrored from the Coordinator program.
/// Matches `CoordinatorError` in `programs/coordinator/src/lib.rs`.
pub mod coordinator_errors {
    pub const INSUFFICIENT_STAKE: u32 = 6000;
    pub const TASK_NOT_PENDING: u32 = 6001;
    pub const TASK_NOT_COMPLETED: u32 = 6002;
    pub const EXECUTOR_INACTIVE: u32 = 6003;
    pub const PDA_ALREADY_INITIALIZED: u32 = 6004;
    pub const INVALID_STATE_URI: u32 = 6005;
    pub const EXECUTOR_UNAUTHORIZED: u32 = 6006;
    pub const INVALID_STATUS: u32 = 6007;
    pub const STATE_HASH_MISMATCH: u32 = 6008;
}

/// On-chain error codes mirrored from the Dark DAO program.
/// Matches `DaoError` in `programs/dark_dao/src/lib.rs`.
pub mod dao_errors {
    pub const PROPOSAL_NOT_ACTIVE: u32 = 6000;
    pub const VOTING_ENDED: u32 = 6001;
    pub const VOTING_STILL_ACTIVE: u32 = 6002;
    pub const INVALID_STATUS: u32 = 6003;
    pub const UNAUTHORIZED_WORKER: u32 = 6004;
}
