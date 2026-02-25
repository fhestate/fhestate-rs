# FHESTATE Architecture

**Technical blueprint for privacy-preserving computation on Solana.**

[![FHESTATE](https://img.shields.io/badge/FHESTATE-Architecture-8A2BE2?style=for-the-badge&logo=rocket&logoColor=white)](https://github.com/fhestate/fhestate-rs)

---

## Design Navigator

*   **1. System Overview**
    *   [High-Level Diagram](#high-level-architecture)
    *   [Hybrid Model](#system-overview)
*   **2. Core Components**
    *   [FHE Engine](#1-fhe-engine-tfhe-rs)
    *   [Client & Node](#2-fhe-cli-client)
*   **3. Mechanics**
    *   [Data Flow](#data-flow)
    *   [Cryptographic Design](#cryptographic-design)
*   **4. Security & Performance**
    *   [Security Model](#security-model)
    *   [Benchmarks](#performance-considerations)

---

## System Overview

FHEstate implements a **Hybrid Privacy Architecture** that combines the best of two worlds:
1.  **Off-chain FHE computation** (for absolute data privacy).
2.  **On-chain Solana verification** (for immutable audit trails and transparency).

### High-Level Architecture

```mermaid
flowchart TB
    subgraph Client ["🖥️ CLIENT LAYER"]
        direction TB
        Wallet[("Solana Wallet")]
        Encrypt["FHE Encryption"]
        Decrypt["FHE Decryption"]
    end

    subgraph Blockchain ["🔗 BLOCKCHAIN LAYER"]
        Solana[("Solana Devnet")]
        Program["FHE Coordinator Program"]
        PDA["StateContainer PDAs"]
        Tasks["Task Accounts"]
    end

    subgraph Execution ["⚙️ EXECUTION LAYER"]
        Node["FHE Executor Node"]
        Compute["Homomorphic Engine"]
        Cache["Content-Addressed Cache"]
    end

    %% Flows
    Wallet -->|SubmitTask| Program
    Program -->|Initialize| PDA
    PDA -->|Emit Event| Node
    Node -->|Resolve URI| Cache
    Node -->|Run FHE| Compute
    
    %% Return Path
    Compute -->|Post Result| Program
    Program -->|Update| PDA
    Program -->|Settle| Tasks

    %% Styles
    style Client fill:#1e1e1e,stroke:#333,color:#fff
    style Blockchain fill:#1e1e1e,stroke:#14F195,color:#fff
    style Execution fill:#1e1e1e,stroke:#8A2BE2,color:#fff
```

---

## Core Components

### 1. FHE Engine (TFHE-rs)

**Library**: [Zama TFHE-rs v0.7.3](https://github.com/zama-ai/tfhe-rs)

The mathematical core that allows computation on encrypted data. TFHE (Torus Fully Homomorphic Encryption) operates on the LWE (Learning With Errors) hard problem.

*   **IND-CPA Security**: Mathematically proven indistinguishability under chosen-plaintext attack.
*   **Probabilistic Encryption**: Every call to `encrypt()` produces a different ciphertext. The same plaintext `42` encrypted twice yields completely different bit patterns — defeating pattern-matching attacks.
*   **Noise Budget**: Each FHE operation adds cryptographic noise to the ciphertext. If noise exceeds the bootstrapping threshold, decryption fails. TFHE-rs manages this automatically via bootstrapping during operations.
*   **Types used**: `FheUint8` (for demo/string ops), `FheUint32` (primary computation type), `FheUint64` (available).
*   **Operations**: Arithmetic (`+`, `-`, `*`), Bitwise (`AND`, `OR`, `XOR`), Comparison (`lt` cast to integer), and Scalar variants.
*   **Server Key Activation**: The `ServerKey` must be set globally on the thread before any homomorphic operation via `set_server_key()` (or `activate_server_key()` in the SDK). This is a TFHE-rs requirement — the key is stored in thread-local storage.

### 2. `fhe-cli` (Client)

The bridge between the user and the blockchain.

*   **Role**: Encrypts user inputs and manages decryption keys.
*   **Action**: Interacts with the `coordinator` program to submit tasks or initialize state.
*   **Security**: Holds the `ClientKey` (Secret) locally. **Never leaves the device.**

### 3. `fhe-node` (Executor)

The decentralized worker that processes FHE tasks.

*   **Role**: Listens to `coordinator` events and polls for `Pending` tasks and `StateContainer` version bumps.
*   **Poll cycle**: Every `POLL_INTERVAL_SECS` (2s), the `ExecutorService` calls `ChainListener::get_program_accounts()` filtered by the `account:Task` Anchor discriminator to find pending tasks, and `get_state_containers()` filtered by `account:StateContainer` to detect inline input updates.
*   **Dual detection paths**: Task accounts (standard path) and StateContainer version bumps (inline path). For inline tasks, the node fetches the recent transaction from chain to extract the op code from the `submit_input` instruction data.
*   **Action**: Performs "Blind Computation" via `StateTransition::apply()` — loads old state from cache, runs `FheMath::execute_op(op, &old_ct, &input_ct)`, writes new state to `.fhe_cache/`, posts result back on-chain.
*   **Security**: Only holds the `ServerKey` (Public). **Cannot see plaintext.**
*   **Staking**: Executors must stake SOL via `register_executor` to participate. Slashing is triggered by `challenge_task`.
*   **Processed state tracking**: Maintains a `HashMap<Pubkey, u64>` of `state_pda → last_seen_version` to avoid reprocessing the same state update.

---

## Data Flow

### End-to-End Transaction Lifecycle

1.  **Preparation**: User encrypts input via `fhe-cli` → `FheUint32::encrypt(value, &client_key)` → serialized with `bincode` → ~32 KB ciphertext bytes.
2.  **Caching**: CLI stores ciphertext in `.fhe_cache/<sha256>.bin` via `LocalCache::store()`. URI = `local://<sha256_hex>`.
3.  **Dispatch**: `fhe-cli` sends a `submit_task` (standard) or `submit_input` (inline) instruction to the Coordinator. The instruction carries the `input_hash` (SHA256 of ciphertext), the `state_uri`, and the `operation` code.
4.  **Detection**: `fhe-node` polls every 2s. For standard tasks: detects new `Task` account with `Pending` status. For inline: detects `StateContainer.version` increment, then fetches the transaction from chain and parses the `submit_input` instruction data to extract the op code.
5.  **State Resolution**: Node calls `get_account_data(&state_pda)` to fetch the current `StateContainer`. Reads `state_uri` (offset `76`) and `state_hash` (offset `40..72`) from the raw account data.
6.  **Computation**: `StateTransition::apply(&cache, old_state_uri, input_bytes, op)` → loads old state ciphertext from cache → runs `FheMath::execute_op(op, &old_ct, &input_ct)` → serializes result → stores to `.fhe_cache/` → returns `(new_uri, sha256_hash)`.
7.  **Settlement**: Node calls `update_state` or `update_state_pda` on-chain, supplying `previous_state_hash`, `result_hash`, and `result_uri`. The Coordinator enforces `state_container.state_hash == previous_state_hash` before accepting.
8.  **Verification**: User fetches the result ciphertext from `.fhe_cache/` using the `state_uri` from the `StateContainer` PDA, verifies the on-chain hash matches `SHA256(ciphertext_bytes)`, and decrypts locally with `client_key.bin`.

---

## Staking & Governance

FHEstate uses a **Staked-Executor Model** to ensure protocol integrity:

*   **Registration**: Executors call `register_executor` with a SOL stake amount ≥ `registry.min_stake`. The SOL is transferred via CPI to the `Executor` account and locked there.
*   **Attribution**: Each `update_state` call sets `task.executor = executor.owner`, creating an immutable on-chain record of who processed each task.
*   **Slashing**: If an executor provides a fraudulent result or reveal, the original submitter (and only the submitter) can call `challenge_task`. This is enforced on-chain: `require!(task.submitter == challenger.key())`.
*   **Resolution**: Successful challenge immediately transfers `executor.stake` lamports to the challenger via direct lamport manipulation, sets `executor.stake = 0`, `executor.active = false`, and marks the task as `Challenged`.
*   **V1 Limitation**: Challenge resolution is optimistic — the submitter's claim is trusted. Future versions will implement ZK proof arbitration where the node must prove it applied the correct FHE operation.

---

## Cryptographic Design

### 🔑 Key Management

| Key Type | Visibility | Purpose | Size |
| :--- | :--- | :--- | :--- |
| **Client Key** | 🔴 **SECRET** | Encrypt/Decrypt data. Owned by User. | ~10 MB |
| **Server Key** | 🟢 **PUBLIC** | Perform Homomorphic Math. Owned by Node. | ~100 MB |

### 🛡️ Encryption Specs

*   **Scheme**: TFHE (Torus Fully Homomorphic Encryption)
*   **Security**: 128-bit quantum-secure (lattice-based).
*   **Expansion**: 1 byte plaintext ≈ 4 KB ciphertext (~4000x expansion).

### 🗄️ Content-Addressed Cache

FHESTATE uses a local file-based, content-addressed storage system (`LocalCache`) for ciphertexts:

*   **Address = Content**: Ciphertext files are named by their SHA256 hash — `<sha256_hex>.bin`. The same ciphertext always maps to the same filename. Corruption is detectable by recomputing the hash.
*   **URI Scheme**: `local://<64-char-sha256-hex>` for local files, `ipfs://<cid>` for IPFS (simulated in v0.1.0).
*   **Full hash**: URIs encode the full 32-byte (64-char hex) SHA256 — no truncation — to minimize collision risk.
*   **On-chain anchor**: Only the URI string and SHA256 hash are stored in the `StateContainer` PDA on-chain. The actual ciphertext (32 KB+) lives in the local cache, keeping transaction costs minimal.

We use **SHA256** hashes of the ciphertext to create a verifiable link between the on-chain event and the off-chain data.

```rust
// Cryptographic Link
Proof = SHA256( Serialize( Encrypted_Data ) )
```

*   **Deterministic**: Same encrypted input always yields same hash.
*   **Verifiable**: Anyone with the ciphertext can verify it matches the on-chain hash.

---

## Security Model

### Threat Analysis

| Threat | Status | Mitigation |
| :--- | :--- | :--- |
| **Data Leakage** | ✅ **Solved** | Data is always encrypted (FHE). Node sees only lattice noise — mathematically indistinguishable from random. |
| **Tampering** | ✅ **Solved** | Solana provides immutable timestamp, ordering, and signature verification. |
| **Fake Results** | ✅ **Solved** | SHA256 hash chain: `state_hash` on-chain must match `SHA256(result_ciphertext_bytes)`. User can verify locally before decrypting. |
| **Replay Attacks** | ✅ **Solved** | `previous_state_hash` must match current on-chain hash. Solana blockhash expiry (5 min) prevents stale transaction replay. |
| **State Rollback** | ✅ **Solved** | `StateContainer.version` is monotonically increasing. Hash chain means you cannot revert to an earlier state without breaking the chain. |
| **Key Theft (Server)** | ✅ **By Design** | Server key is public — possessing it only lets you run FHE ops, not decrypt anything. |
| **Key Theft (Client)** | ⚠️ **User Risk** | Users must protect `client_key.bin`. Loss of this key means permanent data loss. No recovery mechanism exists. |
| **Malicious Node** | ⚠️ **Partial** | Node can submit wrong results but staking/slashing disincentivizes this. ZK proof of correct execution is a roadmap item. |

---

## Performance Considerations

Benchmarks run on standard consumer hardware (M1/M2 Class).

### FHE Operations (Time per Ops)

| Operation | Time (ms) | Notes |
| :--- | :--- | :--- |
| **Encrypt `u8`** | `52ms` | Fast enough for interactive CLI |
| **Decrypt `u8`** | `48ms` | Instant for user |
| **Add `u8 + u8`** | `103ms` | Homomorphic Addition |
| **Mul `u8 * u8`** | `850ms` | Homomorphic Multiplication (Expensive) |

### Blockchain Latency

*   **Solana Block Time**: ~400ms
*   **Confirmation**: ~1-2 seconds (Finalized)
*   **Total Round Trip**: ~5-10 seconds (Network dependent)

---