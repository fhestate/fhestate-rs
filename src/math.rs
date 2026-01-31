use crate::constants::ops;
use sha2::{Digest, Sha256};
use tfhe::prelude::*;
use tfhe::{ClientKey, FheUint32, FheUint64, FheUint8};

/// Homomorphic math operations wrapper.
/// All operations are performed on encrypted data without decryption.
pub struct FheMath;

impl FheMath {
    // ═══════════════════════════════════════════════════════════════════
    // ARITHMETIC OPERATIONS (FheUint32)
    // ═══════════════════════════════════════════════════════════════════

    #[inline]
    pub fn add(a: &FheUint32, b: &FheUint32) -> FheUint32 {
        a + b
    }

    #[inline]
    pub fn sub(a: &FheUint32, b: &FheUint32) -> FheUint32 {
        a - b
    }

    #[inline]
    pub fn mul(a: &FheUint32, b: &FheUint32) -> FheUint32 {
        a * b
    }

    #[inline]
    pub fn bitand(a: &FheUint32, b: &FheUint32) -> FheUint32 {
        a & b
    }

    #[inline]
    pub fn bitor(a: &FheUint32, b: &FheUint32) -> FheUint32 {
        a | b
    }

    #[inline]
    pub fn bitxor(a: &FheUint32, b: &FheUint32) -> FheUint32 {
        a ^ b
    }

    // ═══════════════════════════════════════════════════════════════════
    // SCALAR OPERATIONS
    // ═══════════════════════════════════════════════════════════════════

    #[inline]
    pub fn add_scalar(a: &FheUint32, s: u32) -> FheUint32 {
        a + s
    }

    #[inline]
    pub fn sub_scalar(a: &FheUint32, s: u32) -> FheUint32 {
        a - s
    }

    #[inline]
    pub fn mul_scalar(a: &FheUint32, s: u32) -> FheUint32 {
        a * s
    }

    // ═══════════════════════════════════════════════════════════════════
    // ENCRYPTION / DECRYPTION
    // ═══════════════════════════════════════════════════════════════════

    pub fn encrypt_u8(val: u8, ck: &ClientKey) -> FheUint8 {
        FheUint8::encrypt(val, ck)
    }

    pub fn decrypt_u8(ct: &FheUint8, ck: &ClientKey) -> u8 {
        ct.decrypt(ck)
    }

    pub fn encrypt_u32(val: u32, ck: &ClientKey) -> FheUint32 {
        FheUint32::encrypt(val, ck)
    }

    pub fn decrypt_u32(ct: &FheUint32, ck: &ClientKey) -> u32 {
        ct.decrypt(ck)
    }

    pub fn encrypt_u64(val: u64, ck: &ClientKey) -> FheUint64 {
        FheUint64::encrypt(val, ck)
    }

    pub fn decrypt_u64(ct: &FheUint64, ck: &ClientKey) -> u64 {
        ct.decrypt(ck)
    }

    // ═══════════════════════════════════════════════════════════════════
    // UTILITY FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════

    /// Compute SHA256 hash of bytes.
    pub fn hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let mut out = [0u8; 32];
        out.copy_from_slice(&hasher.finalize());
        out
    }

    /// Compute SHA256 hash and return as hex string.
    pub fn hash_hex(data: &[u8]) -> String {
        hex::encode(Self::hash(data))
    }

    /// Serialize FheUint32 to bytes.
    pub fn serialize_u32(ct: &FheUint32) -> Vec<u8> {
        bincode::serialize(ct).unwrap_or_default()
    }

    /// Deserialize FheUint32 from bytes.
    pub fn deserialize_u32(data: &[u8]) -> Option<FheUint32> {
        bincode::deserialize(data).ok()
    }

    /// Execute operation by code.
    pub fn execute_op(op: u8, a: &FheUint32, b: &FheUint32) -> Option<FheUint32> {
        match op {
            ops::ADD => Some(Self::add(a, b)),
            ops::SUB => Some(Self::sub(a, b)),
            ops::MUL => Some(Self::mul(a, b)),
            ops::AND => Some(Self::bitand(a, b)),
            ops::OR => Some(Self::bitor(a, b)),
            ops::XOR => Some(Self::bitxor(a, b)),
            _ => None,
        }
    }
}
