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

    /// Performs logical AND on two encrypted boolean values (0 or 1).
    #[inline]
    pub fn and(a: &FheUint32, b: &FheUint32) -> FheResult<FheUint32> {
        let result = a & b;
        Ok(result)
    }

    /// Performs logical OR on two encrypted boolean values (0 or 1).
    #[inline]
    pub fn or(a: &FheUint32, b: &FheUint32) -> FheResult<FheUint32> {
        let result = a | b;
        Ok(result)
    }

    /// Performs logical negation on an encrypted boolean value (0 or 1).
    /// Returns 1 if input is 0, and 0 if input is 1.
    /// This is a real FHE operation using tfhe-rs primitives.
    #[inline]
    pub fn not(a: &FheUint32) -> FheResult<FheUint32> {
        // More efficient FHE implementation for NOT (x == 0)
        let result: FheUint32 = a.eq(0u32).cast_into();
        Ok(result)
    }

    /// Production-grade homomorphic multiplexer.
    /// Selects between then_val and else_val based on an encrypted condition cond (0 or 1).
    #[inline]
    pub fn if_then_else(cond: &FheUint32, then_val: &FheUint32, else_val: &FheUint32) -> FheResult<FheUint32> {
        // Convert the condition FheUint32 (0/1) to a Boolean ciphertext for selection
        let bool_cond = cond.ne(0u32);
        let result = bool_cond.if_then_else(then_val, else_val);
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
    fn test_logical_and() {
        let ck = setup();
        let a = enc(1, &ck);
        let b = enc(0, &ck);
        assert_eq!(dec(&FheLogic::and(&a, &a).unwrap(), &ck), 1);
        assert_eq!(dec(&FheLogic::and(&a, &b).unwrap(), &ck), 0);
    }

    #[test]
    #[ignore = "requires full FHE keygen — run with: cargo test -- --ignored"]
    fn test_logical_or() {
        let ck = setup();
        let a = enc(1, &ck);
        let b = enc(0, &ck);
        assert_eq!(dec(&FheLogic::or(&a, &b).unwrap(), &ck), 1);
        assert_eq!(dec(&FheLogic::or(&b, &b).unwrap(), &ck), 0);
    }

    #[test]
    #[ignore = "requires full FHE keygen — run with: cargo test -- --ignored"]
    fn test_logical_not_real() {
        let ck = setup();
        let zero = enc(0, &ck);
        let one = enc(1, &ck);
        assert_eq!(dec(&FheLogic::not(&zero).unwrap(), &ck), 1);
        assert_eq!(dec(&FheLogic::not(&one).unwrap(), &ck), 0);
    }

    #[test]
    #[ignore = "requires full FHE keygen — run with: cargo test -- --ignored"]
    fn test_if_then_else_real() {
        let ck = setup();
        let cond_true = enc(1, &ck);
        let cond_false = enc(0, &ck);
        let a = enc(12345, &ck);
        let b = enc(67890, &ck);
        
        assert_eq!(dec(&FheLogic::if_then_else(&cond_true, &a, &b).unwrap(), &ck), 12345);
        assert_eq!(dec(&FheLogic::if_then_else(&cond_false, &a, &b).unwrap(), &ck), 67890);
    }

    #[test]
    #[ignore = "requires full FHE keygen — run with: cargo test -- --ignored"]
    fn test_production_nested_branching() {
        let ck = setup();
        let c1 = enc(0, &ck); // false
        let c2 = enc(1, &ck); // true
        let a = enc(100, &ck);
        let b = enc(200, &ck);
        let c = enc(300, &ck);

        // Logic: if c1 { a } else if c2 { b } else { c }
        let inner = FheLogic::if_then_else(&c2, &b, &c).unwrap();
        let outer = FheLogic::if_then_else(&c1, &a, &inner).unwrap();
        
        assert_eq!(dec(&outer, &ck), 200);
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
            ("and", FheLogic::and),
            ("or",  FheLogic::or),
        ];
        assert_eq!(binary_ops.len(), 10);

        type UnaryFn = fn(&FheUint32) -> FheResult<FheUint32>;
        let unary_ops: &[(&str, UnaryFn)] = &[
            ("not", FheLogic::not),
        ];
        assert_eq!(unary_ops.len(), 1);

        let scalar_ops: &[(&str, ScalarFn)] = &[
            ("eq_scalar", FheLogic::eq_scalar),
            ("gt_scalar", FheLogic::gt_scalar),
            ("lt_scalar", FheLogic::lt_scalar),
        ];
        assert_eq!(scalar_ops.len(), 3);
    }
}
