# 📚 FHESTATE API

**The definitive reference for building private Solana applications.**

[![FHESTATE](https://img.shields.io/badge/FHESTATE-API-8A2BE2?style=for-the-badge&logo=rocket&logoColor=white)](https://github.com/fhestate/fhestate-rs)
[![Rust](https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Network](https://img.shields.io/badge/Solana-Devnet-14F195?style=for-the-badge&logo=solana&logoColor=black)](https://solana.com)

---

## API Navigator

*   **1. CLI Tools**
    *   [`fhe_proof`](#fhe_proof-local-demo--keygen) - Local verification & Keygen
    *   [`fhe-cli`](#fhe-cli-solana-submission) - Submit tasks to Solana
    *   [`fhe-node`](#fhe-node-background-service) - Background compute service

*   **2. Rust SDK**
    *   [`KeyManager`](#keymanager) - Lifecycle management
    *   [`FheMath`](#fhemath) - Crypto-math engine
    *   [`Core Types`](#core-types) - `FheUint8` and more

*   **3. System Integrations**
    *   [`RPC Endpoints`](#rpc-endpoints) - `getTransaction` response format
    *   [`Error Codes`](#error-codes) - Debugging guide

---

## 1. CLI Commands

### `fhe_proof` (Local Demo & Keygen)

The primary binary for generating keys and verifying FHE logic locally.

#### 🔐 `keygen` - Generate FHE Keys

Generates the Client Key (Secret) and Server Key (Public).

```bash
cargo run --release --bin fhe_proof -- keygen [OPTIONS]
```

**Options:**
- `--out-dir <DIR>` - Output directory for keys (default: `./fhe_keys`)

**Example:**
```bash
cargo run --release --bin fhe_proof -- keygen --out-dir ./keys
```

**Output Files:**
- `client_key.bin`: 🔒 **SECRET**. Used to encrypt/decrypt.
- `server_key.bin`: 🌍 **PUBLIC**. Used by nodes to compute.

---

#### 🔬 `demo` - Run Verification Demo

Runs a full encryption -> compute -> decryption cycle and outputs transparency hashes.

```bash
cargo run --release --bin fhe_proof -- demo
```

**Real Output (Verified):**
```text
═══════════════════════════════════════════════════════════
     FHE STATE: PRODUCTION DEMO
     Target: 'SKD is ready'
═══════════════════════════════════════════════════════════

--- Ciphertext Hashes (SHA256) ---
'S' -> cc9f8376ad33bc930d6d76bbdd130556d5e29790fa35f2a52d8f86d70c19b7df
'K' -> 4d1888d09a796c7c07d3032b79f329bc845c8b2bb07790b0163f1831611ec409
'D' -> 66b88f896547651803e71b92419255015fe6ff7770c4cbecdf482570b2eb50fe
' ' -> b3f6090e6268a93153b1fa5601870e9cd16d29289d810436e44922b899b71d59
'i' -> 51310798c22f274616fb3f6cd39b77bc6876b7a6c40ce3b7356583537599a3d2
's' -> 7234bf420db8ae1633f7754686e97009bdc0444d9b040c7fcd1b7434c50cd71a
' ' -> a972d7dbed9b0993a45cee4885bede29f15e8f6c19af08bd507eadd97cacfd6a
'r' -> 55c9a35e1bb4219604c70eaf53afb78081b09bab22bde61f5c576473d9bf2d1f
'e' -> a33151686e1ecce6a54383989d9b981c56305926fa3d95885c74b9c6228cf4db
'a' -> 5ebbe05ad5bda8ab3783a7a49c9148ceb2e1d0ed50b51fc785c0f8b3cb13a4fd
'd' -> 961f7e235a1e20e80205781f674396fdb335168894b445263dc669e338af060a
'y' -> abe1852a37b8215fcd143f60e278f831a649a3a74601fee33abf0939c0ef9a4d
-------------------------

   Original:  SKD is ready
   Decrypted: TLE!jt!sfbez
   
   STATUS: ✅ VERIFIED SUCCESS
```

---

### `fhe-cli` (Solana Submission)

Tool for interacting with the Solana blockchain.

#### 📡 `submit` - Post FHE Task

Submits an encrypted task to the Coordinator program (SPL Memo).

```bash
cargo run --bin fhe-cli -- submit [OPTIONS]
```

**Options:**
- `--rpc-url <URL>`: Solana RPC (Default: `https://api.devnet.solana.com`)
- `--program <ID>`: Program ID (Default: `MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr`)
- `--wallet <PATH>`: Keypair file (Default: `./deploy-wallet.json`)
- `--op <NUM>`: Operation ID (Default: `1` = SUB). See op codes below.
- `--value <NUM>`: Plaintext `u32` value to encrypt and submit

**Operation codes:**

| Code | Name | Description |
|------|------|-------------|
| `0` | `ADD` | Homomorphic addition |
| `1` | `SUB` | Homomorphic subtraction (default) |
| `2` | `MUL` | Homomorphic multiplication (~800ms) |
| `3` | `CMP` | Returns encrypted `1` if `a < b`, else `0` |
| `4` | `AND` | Bitwise AND |
| `5` | `OR`  | Bitwise OR |
| `6` | `XOR` | Bitwise XOR |

**Real Output:**
```text
[INFO] Submitting FHE Task to Solana
[INFO]    Submitter: 69ZLYxGHckZDCBaDfzp5qh444wQXdETGTKPoetVAdBkW
[INFO]    Sending Transaction...
[INFO]    Success! Transaction Hash: 4w9MESyqbMTkvNZAVn1uLBz1tD8onSuwEqh4yjaxrZLaUvKM7Wf63etQcjvC6XMuRso7auGpH6chFQC6YGyAJ41f
```

---

#### 💼 `wallet` - Utilities

```bash
# Create new wallet
cargo run --bin fhe-cli -- wallet create

# Check balance
cargo run --bin fhe-cli -- wallet balance

# Request Airdrop (Devnet)
cargo run --bin fhe-cli -- wallet airdrop --amount 2
```

---

### `fhe-node` (Background Service)

The background processor that listens for Solana transactions and executes FHE logic.

```bash
cargo run --release --bin fhe-node
```

**Key Responsibilities:**
- 📡 Monitor Solana Devnet for specific program instructions.
- 📂 Retrieve encrypted data from cache or chain.
- ⚙️ Execute homomorphic operations using `server_key.bin`.
- 📦 Post result proofs back to the blockchain.

---

## Rust SDK API

### 🧩 Core Modules

The SDK exports high-level modules for managing the complete FHE lifecycle.

#### `KeyManager`
*(Location: `src/keys.rs`)*

Handles generation, storage, and activation of TFHE keys.

```rust
use fhestate_rs::KeyManager;

// 1. Generate new keys (CPU Intensive: ~30-60s)
let keys = KeyManager::generate()?;

// 2. Save to disk
keys.save("./fhe_keys")?;

// 3. Load from disk
let keys = KeyManager::load("./fhe_keys")?;

// 4. Activate Server Key (Required for computation)
keys.activate(); 
```

#### `FheMath`
*(Location: `src/math.rs`)*

The crypto-math engine. Supports operations on `FheUint8`, `FheUint32`, and `FheUint64`.

**Supported Operations:**

| Op | Description | Example |
|----|-------------|---------|
| `add` | Homomorphic Addition | `FheMath::add(&a, &b)` |
| `sub` | Homomorphic Subtraction | `FheMath::sub(&a, &b)` |
| `mul` | Homomorphic Multiplication | `FheMath::mul(&a, &b)` |
| `cmp` | Less-than: encrypted `1` if `a < b`, else `0` | `FheMath::cmp(&a, &b)` |
| `bitand`| Bitwise AND | `FheMath::bitand(&a, &b)` |
| `bitor` | Bitwise OR | `FheMath::bitor(&a, &b)` |
| `bitxor`| Bitwise XOR | `FheMath::bitxor(&a, &b)` |
| `add_scalar` | Add plaintext `u32` to ciphertext | `FheMath::add_scalar(&a, 10)` |
| `sub_scalar` | Subtract plaintext `u32` | `FheMath::sub_scalar(&a, 5)` |
| `mul_scalar` | Multiply by plaintext `u32` | `FheMath::mul_scalar(&a, 3)` |
| `execute_op` | Dispatch by op code (used by node internally) | `FheMath::execute_op(0, &a, &b)` |

**Encryption & Decryption Helpers:**
```rust
use fhestate_rs::FheMath;

// Encrypt
let ct_8  = FheMath::encrypt_u8(42, &client_key);
let ct_32 = FheMath::encrypt_u32(1000, &client_key);
let ct_64 = FheMath::encrypt_u64(9999999, &client_key);

// Decrypt
let val_8  = FheMath::decrypt_u8(&ct_8, &client_key);
let val_32 = FheMath::decrypt_u32(&ct_32, &client_key);
let val_64 = FheMath::decrypt_u64(&ct_64, &client_key);
```

**Serialization & Hashing:**
```rust
// Serialize FheUint32 ciphertext to bytes (for cache storage)
let bytes: Vec<u8> = FheMath::serialize_u32(&ct_32)?;

// Deserialize from bytes (used by node when loading from cache)
let ct: FheUint32 = FheMath::deserialize_u32(&bytes)?;

// Compute SHA256 proof hash (posted to chain as state_hash)
let hash: [u8; 32] = FheMath::hash(&bytes);
let hash_hex: String = FheMath::hash_hex(&bytes); // 64-char hex string
```

#### `LocalCache`
*(Location: `src/cache.rs`)*

File-based, content-addressed ciphertext cache. Stores ciphertexts as `<sha256>.bin` files under `.fhe_cache/` using SHA256 of the content as the filename (full 32-byte hash → 64-char hex). Returns `local://<hash>` URIs which are posted on-chain as the `state_uri`. Also supports resolving `ipfs://` URIs via a simulated IPFS gateway (real IPFS node integration planned).

```rust
use fhestate_rs::LocalCache;

let cache = LocalCache::new(".fhe_cache");

// Store ciphertext bytes — returns content-addressed URI
let uri = cache.store(&ciphertext_bytes)?;
// uri = "local://a3f9b2c1..." (64-char sha256 hex)

// Load by URI
let bytes = cache.load(&uri)?;

// Check existence
let exists = cache.exists(&uri);

// Resolve any URI scheme (local:// or ipfs://)
let bytes = cache.resolve(&uri)?;
```

---

---

### Core Types

Standardized types used across the FHESTATE ecosystem.

| Type | Bits | FHE Equivalent | Ciphertext Size | Description |
| :--- | :--- | :--- | :--- | :--- |
| `u8` | 8 | `FheUint8` | ~4 KB | Single byte — used for demo (string encryption) |
| `u32` | 32 | `FheUint32` | ~32 KB | Primary computation type — all `FheMath` ops use this |
| `u64` | 64 | `FheUint64` | ~64 KB | Available for encrypt/decrypt; higher latency |

> **Note:** `FheUint16` is not currently implemented. All homomorphic math operations in `FheMath` (`add`, `sub`, `mul`, `bitand`, `bitor`, `bitxor`, `cmp`, scalar ops) operate on `FheUint32`. The `execute_op()` dispatch function accepts an op code byte and two `FheUint32` ciphertexts.

#### `StateTransition`
*(Location: `src/state.rs`)*

The off-chain FHE state machine. Used internally by `fhe-node` to apply homomorphic operations to a user's encrypted state. Handles bootstrapping fresh accounts, loading existing state from cache, applying the op, storing the new state, and returning the SHA256 proof hash.

```rust
use fhestate_rs::StateTransition;

// Apply op to existing state (or bootstrap if state_uri is None)
let (new_uri, result_hash) = StateTransition::apply(
    &cache,
    Some("local://a3f9b2..."),  // current state URI (None for fresh account)
    &input_ciphertext_bytes,    // serialised FheUint32
    ops::ADD,                   // operation code
)?;
// new_uri     = "local://<new_sha256>" — stored in StateContainer.state_uri
// result_hash = [u8; 32] SHA256 — posted to chain as StateContainer.state_hash
```

**Fresh account bootstrap**: When `state_uri` is `None`, the input ciphertext itself becomes the initial state (no operation is applied). This sets up the state for the first real computation.

#### `FheError` / `FheResult`
*(Location: `src/errors.rs`)*

All SDK functions return `FheResult<T>` which is `Result<T, FheError>`. Error variants:

| Variant | Meaning |
|---------|---------|
| `KeyNotFound(path)` | Key file does not exist at path |
| `CacheMiss(uri)` | URI not found in local cache |
| `InvalidOperation(op)` | Unknown op code byte passed to `execute_op` |
| `ComputationFailed(msg)` | FHE operation error (e.g. empty input) |
| `Serialization(e)` | `bincode` serialize/deserialize error |
| `Io(e)` | File system error |
| `RpcError(msg)` | Solana RPC call failed |
| `TransactionFailed(msg)` | Solana transaction rejected |
| `TaskTimeout(secs)` | Task exceeded `TASK_TIMEOUT_SECS` (600s) |

#### `submit_task`
Submit an FHE task to the Solana blockchain.

```rust
pub fn submit_task(
    rpc_url: &str,
    program_id: &str,
    wallet_path: &str,
    op: u8
) -> Result<(), Box<dyn Error>>
```

#### `generate_proof`
Generate SHA256 hash proof for any ciphertext.

```rust
pub fn generate_proof(ciphertext: &FheUint8) -> Result<[u8; 32], Error>
```

---

## Constants Reference

Defined in `src/constants.rs`. Import with `use fhestate_rs::constants::*;`

| Constant | Value | Description |
|----------|-------|-------------|
| `SECURITY_LEVEL` | `128` | FHE security parameter in bits |
| `DEFAULT_RPC` | `https://api.devnet.solana.com` | Default Solana RPC endpoint |
| `KEY_DIR` | `fhe_keys` | Default key storage directory |
| `CACHE_DIR` | `.fhe_cache` | Default ciphertext cache directory |
| `TASK_TIMEOUT_SECS` | `600` | Max seconds before a task is considered timed out |
| `POLL_INTERVAL_SECS` | `2` | Node polling interval in seconds |
| `CT_U8_SIZE` | `8192` | Estimated `FheUint8` ciphertext size in bytes |
| `CT_U32_SIZE` | `32768` | Estimated `FheUint32` ciphertext size in bytes |

---

FHESTATE interacts with Solana using standard JSON-RPC.

### `getTransaction` Response (Full)

When verifying an FHE task on-chain:

```json
{
  "jsonrpc": "2.0",
  "result": {
    "slot": 274381920,
    "transaction": {
      "signatures": [
        "4w9MESyqbMTkvNZAVn1uLBz1tD8onSuwEqh4yjaxrZLaUvKM7Wf63etQcjvC6XMuRso7auGpH6chFQC6YGyAJ41f"
      ],
      "message": {
        "accountKeys": [
            "69ZLYxGHckZDCBaDfzp5qh444wQXdETGTKPoetVAdBkW",
            "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"
        ],
        "instructions": [
          {
            "programIdIndex": 1,
            "accounts": [],
            "data": "RkhFX1RBU0tfU1VCTUlTU0lPTjogT1A9MQ==", 
            "//comment": "Base64 for 'FHE_TASK_SUBMISSION: OP=1'"
          }
        ]
      }
    }
  }
}
```

---

## Error Codes

Common errors encountered during FHE operations.

| Code | Error Name | Description | Solution |
|------|------------|-------------|----------|
| **100** | `KeyNotFound` | Client/Server key missing | Run `fhe_proof -- keygen` |
| **101** | `DecryptionFailed` | Wrong key used for decryption | Ensure `client_key.bin` matches data |
| **200** | `RpcError` | Solana network unreachable | Check internet or change RPC URL |
| **201** | `InsufficientFunds` | Wallet has < 0.001 SOL | Run `wallet airdrop` |
| **202** | `ProgramError` | On-chain instruction failed | Check Program ID and Operation Code |

---