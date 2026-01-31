//! FHEstate SDK - Core Library
//!
//! Provides cryptographic primitives and key management for
//! Fully Homomorphic Encryption on Solana.

pub mod cache;
pub mod constants;
pub mod errors;
pub mod keys;
pub mod math;

pub use cache::LocalCache;
pub use errors::{FheError, FheResult};
pub use keys::{activate_server_key, load_client_key, load_server_key, KeyManager};
pub use math::FheMath;
