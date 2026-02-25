# ❔ FHESTATE FAQ

**The definitive technical resource for Fully Homomorphic Encryption on Solana.**

[![FHESTATE](https://img.shields.io/badge/FHESTATE-Knowledge-8A2BE2?style=for-the-badge&logo=rocket&logoColor=white)](https://github.com/fhestate/fhestate-rs)
---

## 🗺️ Expert Navigator

*   **1. Protocol Fundamentals**
    *   [Q1. What is FHE?](#q1-what-is-fhe)
    *   [Q2. Why FHE on Solana?](#q2-why-fhe-on-solana)
    *   [Q3. Is this production-ready?](#q3-is-this-production-ready)
*   **2. Technical Deep-Dive**
    *   [Q4. Ciphertext Spatial Overhead](#q4-how-large-are-fhe-ciphertexts)
    *   [Q5. Performance Benchmarks](#q5-how-fast-are-fhe-operations)
    *   [Q6. Understanding Bootstrapping](#q6-what-is-bootstrapping-and-why-is-it-slow)
    *   [Q7. Probabilistic Encryption](#q7-is-encryption-deterministic)
*   **3. Security & Trust Model**
    *   [Q8. Patterns vs. Privacy](#q8-why-do-i-see-a-pattern-in-demoregistered-output)
    *   [Q9. Validator Privacy](#q9-can-blockchain-validators-see-my-data)
    *   [Q10. Finality of Key Loss](#q10-can-i-recover-my-clientkeybin)
    *   [Q11. Identity vs. Data Privacy](#q11-does-fhestate-hide-my-wallet-address)
*   **4. Comparisons & Architecture**
    *   [Q12. FHE vs. ZK Proofs](#q12-fhestate-vs-zk-proofs)
    *   [Q13. FHE vs. SGX Enclaves](#q13-fhestate-vs-secure-enclaves-sgx)
    *   [Q14. Error Troubleshooting](#q14-error-code-quick-fix)
    *   [Q15. The Roadmap](#q15-whats-next-for-fhestate)
    *   [Q16. How does state chaining work?](#q16-how-does-the-state-hash-chain-work)

---

## 🏗️ 1. Protocol Fundamentals

### Q1. What is FHE?
**Fully Homomorphic Encryption (FHE)** is the "Holy Grail" of cryptography. It allows code to run on data while it is still encrypted. 
*   **Traditional Crypto**: You encrypt data to store it, but must decrypt it to use it (making it vulnerable).
*   **FHESTATE**: Data stays encrypted even during the math. The server (Node) calculates results blindfolded, and only you (the key owner) can lift the blindfold.

### Q2. Why FHE on Solana?
Solana is uniquely suited to anchor FHE computation due to several properties:
- **Throughput**: 65,000+ TPS means the chain can handle high-frequency state update proofs without becoming a bottleneck.
- **Latency**: ~400ms block time and 1-2s finality means the node gets confirmation quickly after posting a result hash.
- **Cost**: FHE metadata (hashes, URIs, state containers) needs persistent on-chain storage. Solana's rent-based storage model makes storing small `StateContainer` PDAs (~200 bytes) economically viable — orders of magnitude cheaper than Ethereum.
- **PDAs**: Program Derived Addresses give every user a deterministic, permissionless storage slot for their encrypted state — no centralized registry needed.

### Q3. Is this production-ready?
**No.** We are in **Public Alpha**. 
Current version is optimized for developer experience and proof-of-concept. Pro-grade deployments will require Hardware Security Module (HSM) integration and a formal code audit.

---

## 🧬 2. Technical Deep-Dive

### Q4. How large are FHE ciphertexts?
FHE uses **Learning With Errors (LWE)** technology. This involves adding mathematical "noise" to protect your data. 
- **The Overhead**: A single 8-bit integer (u8) expands to approximately **4,096 bytes** of ciphertext. 
- **The Tradeoff**: You pay in **Storage** and **Bandwidth** to gain **Absolute Privacy**.

### Q5. How fast are FHE operations?
FHE math is significantly heavier than plaintext math.
| Operation | Latency (Approx) | Explanation |
| :--- | :--- | :--- |
| **Encrypt** | `50ms` | Lattice generation & noise injection |
| **Add** | `100ms` | Direct homomorphic summation |
| **Multiply** | `800ms+` | Requires "Relinearization" and "Bootstrapping" |

### Q6. What is "Bootstrapping" and why is it slow?
Every FHE operation injects a small amount of cryptographic noise into the ciphertext. Think of it like a signal-to-noise ratio that degrades with each computation. If noise accumulates past a threshold, decryption produces garbage.

**Bootstrapping** is the procedure that refreshes this noise budget — essentially running a homomorphic decryption circuit *inside* another ciphertext, without ever decrypting the data. It resets the noise level so further computation remains correct.

In TFHE-rs, bootstrapping happens automatically as part of every gate (the "PBS" — Programmable Bootstrap). This is why even a single `+` operation on `FheUint8` takes ~100ms — it includes multiple bootstrapping rounds under the hood. Multiplication is especially expensive (~800ms) because it requires an additional "relinearization" step to bring the resulting ciphertext back to the correct structure.

### Q7. Is encryption deterministic?
**Absolutely not.** FHESTATE uses **Probabilistic Encryption**. 
If you encrypt the number `5` ten times, you will get ten completely different ciphertexts. This ensures that an attacker cannot "guess" your data by comparing encrypted values to known outputs (IND-CPA Security).

---

## 🛡️ 3. Security & Trust Model

### Q8. Why do I see a pattern in Demo/Registered output?
*(e.g., S -> T, K -> L)*
This is a common point of confusion for new users! 
1.  **Mathematical Correctness**: The Demo is programmed to perform a `+1` shift. The fact that `S` (83) becomes `T` (84) in the **decrypted result** proves that the math worked perfectly.
2.  **Zero Leakage**: The Node that did the shift **never saw the letter S**. It only saw a block of noise. Only **YOU** (the key owner) see the pattern because you hold the `client_key.bin`. 
3.  **Semantic Security**: To any outside observer (attacker or validator), there is zero pattern. The input and output ciphertexts look like purely random bits.

### Q9. Can blockchain validators see my data?
**Never.** Validators only see:
- That you called the FHESTATE program.
- A cryptographic hash (SHA256) of your encrypted state.
Without your `client_key.bin`, the data is mathematically indestructible.

### Q10. Can I recover my `client_key.bin`?
**NO.** There is no "Reset Password". There is no central authority. If you lose your client key, your data is locked in the lattice forever. 
> [!IMPORTANT]
> Always backup your key files off-chain in a secure physical vault or HSM.

### Q11. Does FHESTATE hide my wallet address?
**No.** FHESTATE protects **Data**, not **Metadata**. 
Solana is a public ledger—everyone knows *which wallet* requested a computation. To protect your identity, use a new "burner" wallet for each session.

---

## ⚔️ 4. Comparisons & Architecture

### Q12. FHESTATE vs. ZK Proofs
- **ZK Proofs**: Used to prove a statement is true (e.g., "I have 5 SOL") without showing the data. Great for **Verification**.
- **FHESTATE**: Used to process data (e.g., "Calculate my interest") without seeing the data. Great for **Computation**.
- **The Union**: Future versions of FHESTATE will use ZK to prove the FHE node worked correctly.

### Q13. FHESTATE vs. Secure Enclaves (SGX)
- **SGX**: Relies on trusting hardware manufacturers (Intel/AMD). If a hardware vulnerability is found, the data is leaked.
- **FHESTATE**: Relies on **Pure Mathematics**. Even if the hardware is compromised, the data remains encrypted.

### Q14. Error Code Quick-Fix
| Code | Error | Solution |
| :--- | :--- | :--- |
| **100** | `KeyNotFound` | Run `fhe_proof -- keygen` |
| **201** | `LowBalance` | Airdrop SOL to your `deploy-wallet.json` |
| **500** | `NoiseOverflow` | Reset your ciphertext or perform bootstrapping. |

---

### Q15. What's next for FHESTATE?

The v0.1.0 release establishes the core protocol. Next phases focus on:

- **GPU Acceleration**: Migrating to `tfhe-cuda` for GPU-accelerated bootstrapping — targeting 10-100x speedup on NVIDIA hardware for multiplication specifically.
- **ZK Proof of Correct Execution**: Using a ZK proof to prove the node ran the correct FHE operation, replacing the current optimistic challenge model with cryptographic guarantees.
- **Threshold Decryption**: Splitting `client_key` into shares held by multiple parties — enabling multi-party computation without a single point of trust.
- **IPFS Integration**: Production IPFS/Arweave node integration to replace the current simulated gateway, enabling truly decentralized ciphertext storage.
- **Mainnet Deployment**: Security audit + HSM key management integration before any Mainnet deployment.

---

### Q16. How does the state hash chain work?

Every `StateContainer` PDA maintains a `state_hash` field (SHA256 of the current ciphertext bytes) and a monotonically increasing `version` counter. When the node posts a result:

1. Reads `state_container.state_hash` (the current on-chain hash) → this becomes `previous_state_hash`
2. Computes `SHA256(new_state_ciphertext_bytes)` → this becomes `result_hash`
3. Calls `update_state(previous_state_hash, result_hash, result_uri)`
4. The Coordinator enforces `require!(state_container.state_hash == previous_state_hash)` — stale reads, replays, or tampered state are all rejected with `StateHashMismatch`
5. On success: `state_hash = result_hash`, `version += 1`

This creates a tamper-evident, ordered sequence of state transitions. You can reconstruct the entire computation history by following the chain of hashes backward.

