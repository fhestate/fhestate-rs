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
    *   [`FheProfiler`](#fheprofiler) - Performance benchmarking
    *   [`VotingTally`](#votingtally) - Confidential DAO logic
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

Tool for interacting with the Solana blockchain, configuring settings, and managing client-side keypairs.

#### 📡 `submit` - Post FHE Task

Submits an encrypted task to the Coordinator program (or SPL Memo program in demo mode).

```bash
# Basic usage using configuration defaults or environment overrides
fhe-cli submit --op <OP_CODE> --value <VALUE>

# Specifying explicit network options using global flags
fhe-cli --rpc-url <URL> --program <ID> --wallet <PATH> submit --op <OP_CODE> --value <VALUE>
```

**Global Options:**
* `--rpc-url <URL>` — Solana RPC Endpoint (Default: reads `FHESTATE_RPC` or `.fhestate/config.json`)
* `--program <ID>` — Coordinator Program ID (Default: reads `FHESTATE_PROGRAM_ID` or defaults to SPL Memo)
* `--wallet <PATH>` — Path to Solana keypair JSON file (Default: reads `FHESTATE_WALLET_PATH` or defaults to `deploy-wallet.json`)

**Operation codes:**

| Code | Name | Description |
|------|------|-------------|
| `0` | `ADD` | Homomorphic addition |
| `1` | `SUB` | Homomorphic subtraction |
| `2` | `MUL` | Homomorphic multiplication (~800ms) |
| `10` | `EQ` | Returns encrypted `1` if `a == b`, else `0` |
| `12` | `GT` | Returns encrypted `1` if `a > b`, else `0` |
| `16` | `MAX` | Homomorphic maximum of two ciphertexts |
| `17` | `MIN` | Homomorphic minimum of two ciphertexts |
| `30` | `VOTE_TALLY` | Optimized Tree-Sum for DAO aggregations |
| `31` | `WINNER` | Constant-time winner detection (multiplexed) |

**Real Output:**
```text
[INFO] Submitting FHE Task to Solana
[INFO]    Submitter: 69ZLYxGHckZDCBaDfzp5qh444wQXdETGTKPoetVAdBkW
[INFO]    Sending Transaction...
[INFO]    Success! Transaction Hash: 4w9MESyqbMTkvNZAVn1uLBz1tD8onSuwEqh4yjaxrZLaUvKM7Wf63etQcjvC6XMuRso7auGpH6chFQC6YGyAJ41f
```

---


#### ⚙️ `doctor` - Diagnostics
Checks local FHE keys, network connections, wallet setup, and SOL balance.
```bash
cargo run --release --bin fhe-cli -- doctor
```

#### 📊 `status` - Keys & Cache Summary
Displays current keys, wallet address, execution mode, and local cache overview.
```bash
cargo run --release --bin fhe-cli -- status
```

#### 🛡️ `keygen` - FHE Keys Generation
Generates FHE keys in the keys directory (does not overwrite unless `--force` is used).
```bash
cargo run --release --bin fhe-cli -- keygen [--force]
```

#### 💼 `wallet new` - Create Wallet
Generates a new Solana keypair JSON file.
```bash
cargo run --release --bin fhe-cli -- wallet new [--out <PATH>]
```

#### 💰 `balance` - Check SOL
Displays the current SOL balance of the configured wallet.
```bash
cargo run --release --bin fhe-cli -- balance
```

#### 🪂 `airdrop` - Request SOL
Requests devnet SOL (defaults to `1.0` SOL).
```bash
cargo run --release --bin fhe-cli -- airdrop [SOL_AMOUNT]
```

#### 📜 `history` - Transaction Logs
Shows recent transaction signatures and provides Devnet Solscan explorer links.
```bash
cargo run --release --bin fhe-cli -- history [--limit <NUM>]
```

#### 🗄️ `cache` - Cache Manager
Lists or displays local ciphertext caches.
```bash
# List cached URIs
cargo run --release --bin fhe-cli -- cache list

# Show detail of a cached ciphertext
cargo run --release --bin fhe-cli -- cache show <HASH_OR_URI>
```

#### 🕵️ `watch` - Monitor Wallet Activity
Polls the wallet for new transactions and outputs Solscan links for new activity.
```bash
cargo run --release --bin fhe-cli -- watch [--interval <SECS>] [--limit <NUM>]
```

#### 🔄 `flow counter` - Automated Flow
Initializes a user StateContainer PDA and submits a task sequentially.
```bash
cargo run --release --bin fhe-cli -- flow counter [--value <NUM>]
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
| `tree_sum` | **Optimized $O(\log n)$ aggregation** | `FheMath::tree_sum(vec![a, b, c])` |
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

#### `FheProfiler`
*(Location: `src/profiler.rs`)*

Production-grade benchmarking suite for FHE circuits.

```rust
use fhestate_rs::FheProfiler;

// Benchmark any FHE operation
let result = FheProfiler::benchmark("My Circuit", 10, || {
    FheMath::add(&a, &b)
});

// Print a formatted report
FheProfiler::print_report(&[result]);
```

#### `VotingTally`
*(Location: `src/voting.rs`)*

Confidential voting and winner detection logic for the Dark DAO.

```rust
use fhestate_rs::VotingTally;

// 1. Tally ballots using Tree-Sum
let total = VotingTally::tally_binary_votes(encrypted_votes)?;

// 2. Find encrypted winner from multiple candidate totals
let winner_score = VotingTally::find_winner(&[total_a, total_b])?;
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

## 🔗 On-Chain Smart Contracts & Program IDs

The FHESTATE-rs ecosystem consists of three main Anchor programs on Solana Devnet:

### 1. Shielded Vault Program
*   **Program ID**: `D14VbLLPcqkkZ6p4M9UDs4xfNdtB1tQDUqi7ZTt89etC`
*   **Purpose**: Confidential SOL deposits, blinded transfers, and TEE remote-attestation-authorized withdrawals.

#### Instructions

| Instruction | Accounts | Parameters | Description |
|:---|:---|:---|:---|
| `initialize_vault` | `[registry, authority, system_program]` | `attestation_authority: Pubkey` | Creates global vault registry config |
| `initialize_account`| `[encrypted_account, owner, system_program]` | None | Creates individual user encrypted balance PDA |
| `shield_funds` | `[user, vault, registry, system_program]` | `amount: u64` | Deposits SOL into vault and emits `ShieldEvent` |
| `register_enclave` | `[authority, registry, enclave_account, instructions, system_program]` | `enclave_key: Pubkey` | Attests SGX enclave via precompile sig check |
| `toggle_enclave` | `[authority, registry, enclave_account]` | `is_active: bool` | Enables/disables an enclave |
| `execute_transfer_fhe`| `[authority, registry, sender_account, receiver_account]` | `new_sender_hash: [u8;32]`, `new_receiver_hash: [u8;32]` | Admin-authorized private transfer |
| `execute_transfer_fhe_tee`| `[enclave_signer, enclave_account, sender_account, receiver_account]` | `new_sender_hash: [u8;32]`, `new_receiver_hash: [u8;32]` | TEE-authorized private transfer |
| `unshield_funds` | `[authority, registry, vault, user, system_program]` | `amount: u64`, `vault_bump: u8` | Admin-authorized withdrawal |
| `unshield_funds_tee` | `[enclave_signer, enclave_account, registry, vault, user, system_program]` | `amount: u64`, `vault_bump: u8` | TEE-authorized withdrawal |
| `close_registry` | `[admin, registry]` | None | Closes/reclaims registry PDA rent lamports |

#### State Schemas

```rust
pub struct VaultRegistry {
    pub admin: Pubkey,
    pub attestation_authority: Pubkey,
    pub total_liquidity: u64,
    pub approved_mrenclave: [u8; 32],
}

pub struct EncryptedAccount {
    pub owner: Pubkey,
    pub balance_hash: [u8; 32],
}

pub struct EnclaveAccount {
    pub enclave_key: Pubkey,
    pub is_active: bool,
}
```

---

### 2. Coordinator Program
*   **Program ID**: `57YPM8JYv8t6wArmZTD14PNo6ES9CYKGRYcZWC4FZEnq`
*   **Purpose**: Off-chain FHE execution task queuing, state-hash chaining, and staked executor registration.

#### Instructions

| Instruction | Accounts | Parameters | Description |
|:---|:---|:---|:---|
| `initialize` | `[registry, authority, system_program]` | `min_stake: u64` | Spawns executor registry coordinator |
| `register_executor`| `[registry, executor, owner, system_program]` | `stake_amount: u64` | Registers executor with locked SOL stake |
| `submit_task` | `[registry, task, submitter, system_program]` | `id: u64`, `input_hash: [u8;32]`, `input_uri: String`, `op: u8`, `target_owner: Option<Pubkey>` | Enqueues standard FHE compute task |
| `initialize_state` | `[state_container, submitter, system_program]` | None | Initializes a new `StateContainer` PDA |
| `submit_input` | `[state_container, submitter]` | `encrypted_data: Vec<u8>`, `operation: u8` | Submits inline ciphertext data |
| `update_state` | `[task, executor, state_container, owner]` | `previous_state_hash: [u8;32]`, `result_hash: [u8;32]`, `result_uri: String` | Settles FHE task and updates user's StateContainer hash |
| `update_state_pda` | `[state_container, owner_key, executor, owner]` | `previous_state_hash: [u8;32]`, `result_hash: [u8;32]`, `result_uri: String` | Directly updates state PDA (inline flow) |
| `request_reveal` | `[task, submitter]` | None | Requests plaintext reveal for completed FHE task |
| `provide_reveal` | `[task, executor]` | `reveal_data: String` | Submits plaintext result (reveal phase) |
| `challenge_task` | `[task, executor, challenger]` | None | Slashes executor stake for invalid computation |

#### State Schemas

```rust
pub struct Registry {
    pub authority: Pubkey,
    pub min_stake: u64,
    pub task_count: u64,
    pub executor_count: u64,
}

pub struct Executor {
    pub owner: Pubkey,
    pub stake: u64,
    pub active: bool,
    pub tasks_completed: u64,
}

pub struct Task {
    pub id: u64,
    pub submitter: Pubkey,
    pub target_owner: Pubkey,
    pub input_hash: [u8; 32],
    pub input_uri: String,
    pub operation: u8,
    pub status: TaskStatus,
    pub result_hash: [u8; 32],
    pub result_uri: String,
    pub reveal_result: String,
    pub executor: Pubkey,
}

pub struct StateContainer {
    pub owner: Pubkey,
    pub state_hash: [u8; 32],
    pub state_uri: String,
    pub version: u64,
}
```

---

### 3. Dark DAO Program
*   **Program ID**: `Ay5Z1HQrsfnYNhRt48Mujr7k1b91bV7ir4jATYocVp5s`
*   **Purpose**: Confidential governance, encrypted voting, and homomorphic ballot aggregation.

#### Instructions

| Instruction | Accounts | Parameters | Description |
|:---|:---|:---|:---|
| `initialize` | `[config, authority, system_program]` | None | Initializes DAO configuration |
| `authorize_worker` | `[config, worker_record, worker_key, authority, system_program]` | `worker_key: Pubkey` | Configures authorized FHE workers |
| `create_proposal` | `[proposal, tally, creator, system_program]` | `description: String`, `voting_period: i64` | Creates proposal and matching tally PDA |
| `cast_encrypted_vote`| `[proposal, vote_record, voter, system_program]` | `encrypted_vote: Vec<u8>` | Emits `VoteCast` containing encrypted ballot |
| `update_tally` | `[proposal, tally, worker_record, worker]` | `new_state_hash: [u8;32]`, `new_state_uri: String` | Aggregates votes homomorphically on-chain |
| `finalize_tally` | `[proposal, tally, creator]` | `result_hash: [u8;32]`, `result_uri: String` | Finalizes tally outcome decryption |

#### State Schemas

```rust
pub struct DaoConfig {
    pub authority: Pubkey,
}

pub struct AuthorizedWorker {
    pub pubkey: Pubkey,
    pub is_active: bool,
}

pub struct Proposal {
    pub creator: Pubkey,
    pub description: String,
    pub start_time: i64,
    pub end_time: i64,
    pub status: ProposalStatus,
    pub total_votes: u64,
}

pub struct EncryptedTally {
    pub proposal: Pubkey,
    pub state_hash: [u8; 32],
    pub state_uri: String,
    pub version: u64,
}

pub struct VoteRecord {
    pub voter: Pubkey,
    pub proposal: Pubkey,
    pub timestamp: i64,
}
```

---

## 🛠️ TypeScript / Web3.js Bindings Integration

Clients submit TEE remote-attestation enclaves by appending a preceding Ed25519 precompile instruction to verify the 64-byte payload `[enclave_key (32) | mrenclave (32)]`:

```typescript
import { 
  Ed25519Program, 
  Transaction, 
  PublicKey, 
  SystemProgram 
} from '@solana/web3.js';
import * as nacl from 'tweetnacl';

async function registerEnclave(
  program: any, 
  authority: Keypair, 
  enclaveKey: PublicKey, 
  attestationAuthority: Keypair, 
  mrenclave: Uint8Array
) {
  // 1. Build 64-byte attestation payload
  const message = Buffer.concat([enclaveKey.toBuffer(), mrenclave]);
  
  // Sign attestation payload using Attestation Authority private key
  const signature = nacl.sign.detached(message, attestationAuthority.secretKey);

  // 2. Build the Ed25519 Precompile Instruction
  const ed25519Instruction = Ed25519Program.createInstructionWithPublicKey({
    publicKey: attestationAuthority.publicKey.toBuffer(),
    message: message,
    signature: signature,
  });

  // 3. Build register_enclave Instruction
  const registerIx = await program.methods
    .registerEnclave(enclaveKey)
    .accounts({
      authority: authority.publicKey,
      registry: vaultRegistryPda,
      enclaveAccount: enclavePda,
      instructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
      systemProgram: SystemProgram.programId,
    })
    .instruction();

  // 4. Submit both atomically
  const transaction = new Transaction().add(ed25519Instruction, registerIx);
  await anchor.Provider.env().sendAndConfirm(transaction, [authority]);
}
```

---

## 📜 Error Codes & Diagnostics

Below are the error tables returned by the on-chain programs:

### Shielded Vault Errors (`shielded_vault`)

| Code | Name | Description |
|:---|:---|:---|
| `6000` | `Unauthorized` | Action restricted to admin authority |
| `6001` | `UnauthorizedEnclave` | Signer is not an active TEE enclave |
| `6002` | `InvalidEd25519Instruction` | Missing or invalid preceding Ed25519 precompile |
| `6003` | `InvalidAttestationMessage` | Precompile signed message length is not exactly 64 bytes |
| `6004` | `EnclaveKeyMismatch` | Precompile target enclave key does not match transaction input |
| `6005` | `InvalidMrenclave` | Signed code measurement `MRENCLAVE` hash is not approved |

### Coordinator Errors (`coordinator`)

| Code | Name | Description |
|:---|:---|:---|
| `6000` | `InsufficientStake` | Staked executor SOL is below `min_stake` threshold |
| `6001` | `TaskNotPending` | Operation attempted on non-pending task |
| `6002` | `TaskNotCompleted` | Operation requires a completed task |
| `6003` | `ExecutorInactive` | Executor account is disabled or inactive |
| `6005` | `InvalidStateUri` | URI scheme is not `local://` or `ipfs://` |
| `6006` | `ExecutorUnauthorized` | Signer is not the owner of the executor PDA |
| `6008` | `StateHashMismatch` | Stale state hash; another node updated the state container |

### Dark DAO Errors (`dark_dao`)

| Code | Name | Description |
|:---|:---|:---|
| `6000` | `ProposalNotActive` | Action attempted on inactive proposal |
| `6001` | `VotingEnded` | Proposal voting period has expired |
| `6002` | `VotingStillActive` | Finalization attempted before voting ended |
| `6003` | `InvalidStatus` | Proposal status is not valid for transaction |
| `6004` | `UnauthorizedWorker` | Worker is not registered or active |

### Off-Chain SDK Errors

| Code | Name | Description |
|:---|:---|:---|
| `100` | `KeyNotFound` | Client or server FHE binary key file is missing on disk |
| `101` | `DecryptionFailed` | Decryption failed (caused by mismatched `client_key.bin` keys) |
| `200` | `RpcError` | Solana RPC endpoint unreachable |
| `201` | `InsufficientFunds` | Submitter wallet has less than `0.01` SOL to cover gas fees |
| `202` | `ProgramError` | On-chain instruction failed |

---