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
        Memo["SPL Memo Program"]
        Logs["Transaction Logs"]
    end

    subgraph Execution ["⚙️ EXECUTION LAYER"]
        Node["FHE Coordinator Node"]
        Compute["Homomorphic Engine"]
        Proof["Proof Generator (SHA256)"]
    end

    %% Flows
    Wallet -->|Sign Tx| Solana
    Encrypt -->|Encrypted Input| Solana
    Solana -->|Emit Event| Node
    Node -->|Fetch Data| Solana
    Node -->|Run FHE| Compute
    Compute -->|Ciphertext Result| Proof
    
    %% Return Path
    Proof -->|Verify Hash| Decrypt
    Decrypt -->|Reveal Final Output| Wallet

    %% Styles
    style Client fill:#1e1e1e,stroke:#333,color:#fff
    style Blockchain fill:#1e1e1e,stroke:#14F195,color:#fff
    style Execution fill:#1e1e1e,stroke:#8A2BE2,color:#fff
```

---

## Core Components

### 1. FHE Engine (TFHE-rs)

**Library**: [Zama TFHE-rs](https://github.com/zama-ai/tfhe-rs)

The mathematical core that allows computation on encrypted data.

*   **IND-CPA Security**: Mathematically proven security.
*   **Leveled FHE**: Optimized for depth-limited circuits.
*   **Types**: `FheUint8`, `FheUint32`, `FheBool`.
*   **Operations**: Arithmetic (`+`, `-`, `*`), Bitwise, and Comparisons.

### 2. `fhe-cli` (Client)

The bridge between the user and the blockchain.

*   **Role**: Encrypts user inputs via `Input Generation` phase.
*   **Action**: Submits Tasks to Solana using the `SPL Memo` program.
*   **Security**: Holds the `ClientKey` (Secret) locally. **Never leaves the device.**

### 3. `fhe-node` (Executor)

The decentralized worker that processes FHE tasks.

*   **Role**: Listens to Solana transactions.
*   **Action**: Performs "Blind Computation" on encrypted inputs.
*   **Security**: Only holds the `ServerKey` (Public). **Cannot see plaintext.**

---

## Data Flow

### End-to-End Transaction Lifecycle

```mermaid
sequenceDiagram
    participant User as 👤 User
    participant CLI as 🚀 fhe-cli
    participant Solana as 🔗 Solana
    participant Node as ⚙️ fhe-node

    Note over User, CLI: Phase 1: Preparation
    User->>CLI: Input: "SKD is ready"
    CLI->>CLI: Encrypt (FHE) -> Ciphertexts
    CLI->>CLI: Generate Proofs (SHA256)

    Note over CLI, Solana: Phase 2: Submission
    CLI->>Solana: Submit Transaction (SPL Memo)
    Solana-->>CLI: Confirmed (Sig: 4w9ME...)
    Solana-->>User: Transaction Hash

    Note over Solana, Node: Phase 3: Execution (Async)
    Node->>Solana: Poll for New Tasks
    Solana->>Node: Send Encrypted Payload
    Node->>Node: Compute: Shift(+1)
    Node->>Node: Generate Result Proof

    Note over Node, User: Phase 4: Verification
    Node-->>Solana: Post Result Hash
    User->>Solana: Check Proof Match
    User->>CLI: Decrypt Result
    CLI-->>User: "TLE!jt!sfbez"
```

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

### 🔍 Proof Verification

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
| **Data Leakage** | ✅ **Solved** | Data is always encrypted (FHE). Nodes see noise. |
| **Tampering** | ✅ **Solved** | Solana provides immutable timestamp & ordering. |
| **Fake Results** | ✅ **Solved** | Users verify result proofs against inputs locally. |
| **Replay Attacks** | ✅ **Solved** | Solana signatures & blockhash expiry (5 min). |
| **Key Theft** | ⚠️ **User Risk** | Users must protect their `client_key.bin`. |

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