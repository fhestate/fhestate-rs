//! FHEstate SDK - Core Library
//!
//! Provides cryptographic primitives and key management for
//! Fully Homomorphic Encryption on Solana.

pub mod cache;
pub mod constants;
pub mod errors;
pub mod keys;
pub mod logic;
pub mod math;
pub mod profiler;
pub mod voting;
pub mod state;

pub use cache::LocalCache;
pub use errors::{FheError, FheResult};
pub use keys::{activate_server_key, load_client_key, load_server_key, KeyManager};
pub use logic::FheLogic;
pub use math::FheMath;
pub use profiler::{BenchmarkResult, FheProfiler};
pub use voting::VotingTally;
pub use state::StateTransition;
