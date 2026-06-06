# ❔ FHESTATE FAQ

**The definitive technical resource for Fully Homomorphic Encryption on Solana.**

[![FHESTATE](https://img.shields.io/badge/FHESTATE-Knowledge-8A2BE2?style=for-the-badge&logo=rocket&logoColor=white)](https://github.com/fhestate/fhestate-rs)
---

## 🗺️ Expert Navigator

*   **4. Privacy Operations (Dark DAO)**
    *   [Q15. What is a Dark DAO?](#q15-what-is-a-dark-dao)
    *   [Q16. How does Tree-Sum optimize voting?](#q16-how-does-tree-sum-optimize-voting)
    *   [Q17. The Roadmap](#q17-whats-next-for-fhestate)
    *   [Q18. How does state chaining work?](#q18-how-does-the-state-hash-chain-work)

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
| **100** | `KeyNotFound` | Run `fhe-cli keygen` or `fhe-cli setup` |
| **201** | `LowBalance` | Run `fhe-cli airdrop` or fund the wallet |
| **500** | `NoiseOverflow` | Reset your ciphertext or perform bootstrapping. |

---

## 🏛️ 4. Privacy Operations (Dark DAO)

### Q15. What is a Dark DAO?
A **Dark DAO** is a decentralized autonomous organization where governance is completely confidential. 
- **Encrypted Ballots**: No one can see your individual vote.
- **Blind Tallying**: The result is calculated without the node ever knowing the partial scores.
- **Private Winners**: Only the winning outcome is revealed; the margin of victory remains hidden.

### Q16. How does Tree-Sum optimize voting?
Normally, adding 100 votes in FHE would create a "chain" of 99 additions, causing cryptographic noise to explode. **Tree-Sum** organizes these additions into a binary tree structure.
- **Benefit**: Instead of 99 noise-levels deep, the result is only **7 levels deep** ($\log_2 100$). This ensures accurate decryptions for large-scale governance.

---

### Q17. What's next for FHESTATE?

We are currently focused on productionizing the core infrastructure:

- **GPU Acceleration**: Migrating to `tfhe-cuda` for GPU-accelerated bootstrapping.
- **ZK Proof of Correct Execution**: Proving the FHE node worked correctly without requiring trust.
- **Threshold Decryption**: Splitting keys across multiple nodes for decentralized security.

---

### Q18. How does the state hash chain work?

Every `StateContainer` PDA maintains a `state_hash` field (SHA256 of the current ciphertext bytes) and a monotonically increasing `version` counter. When the node posts a result:

1. Reads `state_container.state_hash` (the current on-chain hash) → this becomes `previous_state_hash`
2. Computes `SHA256(new_state_ciphertext_bytes)` → this becomes `result_hash`
3. Calls `update_state(previous_state_hash, result_hash, result_uri)`
4. The Coordinator enforces `require!(state_container.state_hash == previous_state_hash)` — stale reads, replays, or tampered state are all rejected with `StateHashMismatch`
5. On success: `state_hash = result_hash`, `version += 1`

This creates a tamper-evident, ordered sequence of state transitions. You can reconstruct the entire computation history by following the chain of hashes backward.

