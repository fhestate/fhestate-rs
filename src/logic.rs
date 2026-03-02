//! Encrypted Logic Operations
//!
//! Provides fundamental FHE comparison and branching primitives for `FheUint32`
//! types using the TFHE-rs library. All operations return encrypted results
//! without leaking information about the underlying plaintext values.

use crate::errors::FheResult;
use tfhe::prelude::*;
use tfhe::FheUint32;

/// A collection of static methods for performing encrypted logic and comparisons.
pub struct FheLogic;

impl FheLogic {
    /// Checks if two encrypted values are equal.
    /// Returns an encrypted 1 if true, and 0 otherwise.
    #[inline]
    pub fn eq(a: &FheUint32, b: &FheUint32) -> FheResult<FheUint32> {
        let result: FheUint32 = a.eq(b).cast_into();
        Ok(result)
    }

    /// Checks if two encrypted values are not equal.
    /// Returns an encrypted 1 if true, and 0 otherwise.
    #[inline]
    pub fn ne(a: &FheUint32, b: &FheUint32) -> FheResult<FheUint32> {
        let result: FheUint32 = a.ne(b).cast_into();
        Ok(result)
    }

    /// Checks if the first value is strictly greater than the second.
    /// Returns an encrypted 1 if true, and 0 otherwise.
    #[inline]
    pub fn gt(a: &FheUint32, b: &FheUint32) -> FheResult<FheUint32> {
        let result: FheUint32 = a.gt(b).cast_into();
        Ok(result)
    }

    /// Checks if the first value is strictly less than the second.
    /// Returns an encrypted 1 if true, and 0 otherwise.
    #[inline]
    pub fn lt(a: &FheUint32, b: &FheUint32) -> FheResult<FheUint32> {
        let result: FheUint32 = a.lt(b).cast_into();
        Ok(result)
    }

    /// Checks if the first value is greater than or equal to the second.
    /// Returns an encrypted 1 if true, and 0 otherwise.
    #[inline]
    pub fn ge(a: &FheUint32, b: &FheUint32) -> FheResult<FheUint32> {
        let result: FheUint32 = a.ge(b).cast_into();
        Ok(result)
    }

    /// Checks if the first value is less than or equal to the second.
    /// Returns an encrypted 1 if true, and 0 otherwise.
    #[inline]
    pub fn le(a: &FheUint32, b: &FheUint32) -> FheResult<FheUint32> {
        let result: FheUint32 = a.le(b).cast_into();
        Ok(result)
    }

    /// Returns the maximum of two encrypted values.
    /// This is implemented as a homomorphic multiplexer (if a > b then a else b).
    #[inline]
    pub fn max(a: &FheUint32, b: &FheUint32) -> FheResult<FheUint32> {
        let result = a.gt(b).if_then_else(a, b);
        Ok(result)
    }

    /// Returns the minimum of two encrypted values.
    /// This is implemented as a homomorphic multiplexer (if a < b then a else b).
    #[inline]
    pub fn min(a: &FheUint32, b: &FheUint32) -> FheResult<FheUint32> {
        let result = a.lt(b).if_then_else(a, b);
        Ok(result)
    }

    /// Compares an encrypted value with a plaintext scalar for equality.
    /// Returns an encrypted 1 if equal, 0 otherwise.
    #[inline]
    pub fn eq_scalar(a: &FheUint32, scalar: u32) -> FheResult<FheUint32> {
        let result: FheUint32 = a.eq(scalar).cast_into();
        Ok(result)
    }

    /// Checks if an encrypted value is strictly greater than a plaintext scalar.
    /// Returns an encrypted 1 if true, 0 otherwise.
    #[inline]
    pub fn gt_scalar(a: &FheUint32, scalar: u32) -> FheResult<FheUint32> {
        let result: FheUint32 = a.gt(scalar).cast_into();
        Ok(result)
    }

    /// Checks if an encrypted value is strictly less than a plaintext scalar.
    /// Returns an encrypted 1 if true, 0 otherwise.
    #[inline]
    pub fn lt_scalar(a: &FheUint32, scalar: u32) -> FheResult<FheUint32> {
        let result: FheUint32 = a.lt(scalar).cast_into();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tfhe::{generate_keys, set_server_key, ConfigBuilder, FheUint32};

    /// Sets up a local FHE environment for testing.
    fn setup() -> tfhe::ClientKey {
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);
        set_server_key(server_key);
        client_key
    }

    fn enc(val: u32, ck: &tfhe::ClientKey) -> FheUint32 {
        FheUint32::encrypt(val, ck)
    }

    fn dec(ct: &FheUint32, ck: &tfhe::ClientKey) -> u32 {
        ct.decrypt(ck)
    }

    #[test]
    #[ignore = "requires full FHE keygen — run with: cargo test -- --ignored"]
    fn test_eq_same_values() {
        let ck = setup();
        let a = enc(42, &ck);
        let b = enc(42, &ck);
        let result = FheLogic::eq(&a, &b).unwrap();
        assert_eq!(dec(&result, &ck), 1);
    }

    #[test]
    #[ignore = "requires full FHE keygen — run with: cargo test -- --ignored"]
    fn test_eq_different_values() {
        let ck = setup();
        let a = enc(10, &ck);
        let b = enc(20, &ck);
        let result = FheLogic::eq(&a, &b).unwrap();
        assert_eq!(dec(&result, &ck), 0);
    }

    #[test]
    #[ignore = "requires full FHE keygen — run with: cargo test -- --ignored"]
    fn test_ne() {
        let ck = setup();
        let a = enc(7, &ck);
        let b = enc(8, &ck);
        assert_eq!(dec(&FheLogic::ne(&a, &b).unwrap(), &ck), 1);
        assert_eq!(dec(&FheLogic::ne(&a, &a).unwrap(), &ck), 0);
    }

    #[test]
    #[ignore = "requires full FHE keygen — run with: cargo test -- --ignored"]
    fn test_gt() {
        let ck = setup();
        let large = enc(100, &ck);
        let small = enc(3, &ck);
        assert_eq!(dec(&FheLogic::gt(&large, &small).unwrap(), &ck), 1);
        assert_eq!(dec(&FheLogic::gt(&small, &large).unwrap(), &ck), 0);
    }

    #[test]
    #[ignore = "requires full FHE keygen — run with: cargo test -- --ignored"]
    fn test_lt() {
        let ck = setup();
        let a = enc(5, &ck);
        let b = enc(50, &ck);
        assert_eq!(dec(&FheLogic::lt(&a, &b).unwrap(), &ck), 1);
        assert_eq!(dec(&FheLogic::lt(&b, &a).unwrap(), &ck), 0);
    }

    #[test]
    #[ignore = "requires full FHE keygen — run with: cargo test -- --ignored"]
    fn test_ge_le_boundary() {
        let ck = setup();
        let a = enc(10, &ck);
        let b = enc(10, &ck);
        assert_eq!(dec(&FheLogic::ge(&a, &b).unwrap(), &ck), 1);
        assert_eq!(dec(&FheLogic::le(&a, &b).unwrap(), &ck), 1);
    }

    #[test]
    #[ignore = "requires full FHE keygen — run with: cargo test -- --ignored"]
    fn test_max() {
        let ck = setup();
        let a = enc(77, &ck);
        let b = enc(33, &ck);
        assert_eq!(dec(&FheLogic::max(&a, &b).unwrap(), &ck), 77);
    }

    #[test]
    #[ignore = "requires full FHE keygen — run with: cargo test -- --ignored"]
    fn test_min() {
        let ck = setup();
        let a = enc(5, &ck);
        let b = enc(200, &ck);
        assert_eq!(dec(&FheLogic::min(&a, &b).unwrap(), &ck), 5);
    }

    #[test]
    #[ignore = "requires full FHE keygen — run with: cargo test -- --ignored"]
    fn test_scalar_comparisons() {
        let ck = setup();
        let a = enc(50, &ck);
        assert_eq!(dec(&FheLogic::eq_scalar(&a, 50).unwrap(), &ck), 1);
        assert_eq!(dec(&FheLogic::gt_scalar(&a, 49).unwrap(), &ck), 1);
        assert_eq!(dec(&FheLogic::lt_scalar(&a, 50).unwrap(), &ck), 0);
    }

    #[test]
    #[ignore = "requires full FHE keygen — run with: cargo test -- --ignored"]
    fn test_min_lte_max_consistency() {
        let ck = setup();
        let a = enc(10, &ck);
        let b = enc(20, &ck);
        let min_v = dec(&FheLogic::min(&a, &b).unwrap(), &ck);
        let max_v = dec(&FheLogic::max(&a, &b).unwrap(), &ck);
        assert!(min_v <= max_v);
    }

    /// Structural verification test.
    /// This test ensures that all methods are correctly defined and reachable via function pointers,
    /// without requiring an expensive FHE key generation.
    #[test]
    fn test_method_names_compile() {
        type BinaryFn = fn(&FheUint32, &FheUint32) -> FheResult<FheUint32>;
        type ScalarFn = fn(&FheUint32, u32) -> FheResult<FheUint32>;

        let binary_ops: &[(&str, BinaryFn)] = &[
            ("eq",  FheLogic::eq),
            ("ne",  FheLogic::ne),
            ("gt",  FheLogic::gt),
            ("lt",  FheLogic::lt),
            ("ge",  FheLogic::ge),
            ("le",  FheLogic::le),
            ("max", FheLogic::max),
            ("min", FheLogic::min),
        ];
        assert_eq!(binary_ops.len(), 8);

        let scalar_ops: &[(&str, ScalarFn)] = &[
            ("eq_scalar", FheLogic::eq_scalar),
            ("gt_scalar", FheLogic::gt_scalar),
            ("lt_scalar", FheLogic::lt_scalar),
        ];
        assert_eq!(scalar_ops.len(), 3);
    }
}
