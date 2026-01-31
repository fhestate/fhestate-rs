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
- `--op <NUM>`: Operation ID (Default: `1` for Shift)

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
| `bitand`| Bitwise AND | `FheMath::bitand(&a, &b)` |
| `bitor` | Bitwise OR | `FheMath::bitor(&a, &b)` |
| `bitxor`| Bitwise XOR | `FheMath::bitxor(&a, &b)` |
| `add_scalar` | Add unencrypted integer | `FheMath::add_scalar(&a, 10)` |

**Encryption & Decryption Helpers:**
```rust
use fhestate_rs::FheMath;

// Encrypt
let ct_8  = FheMath::encrypt_u8(42, &client_key);
let ct_32 = FheMath::encrypt_u32(1000, &client_key);

// Decrypt
let val = FheMath::decrypt_u8(&ct_8, &client_key);
```

#### `LocalCache`
*(Location: `src/cache.rs`)*

In-memory storage for testing FHE operations without blockchain latency.

```rust
use fhestate_rs::LocalCache;

let mut cache = LocalCache::new();
cache.insert("user_balance", encrypted_balance);
```

---

---

### Core Types

Standardized types used across the FHESTATE ecosystem.

| Type | Bits | FHE Equivalent | Description |
| :--- | :--- | :--- | :--- |
| `u8` | 8 | `FheUint8` | Single byte encryption (standard) |
| `u16` | 16 | `FheUint16` | 16-bit integer (large) |
| `u32` | 32 | `FheUint32` | 32-bit integer (very large) |
| `u64` | 64 | `FheUint64` | 64-bit integer (extreme latency) |

---

### Functions

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

## RPC Endpoints

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