# 📜 FHESTATE CHANGELOG

**The chronological evolution of confidential computing on Solana.**

[![Version](https://img.shields.io/badge/Version-v0.3.2-8A2BE2?style=for-the-badge&logo=rocket&logoColor=white)](https://github.com/fhestate/fhestate-rs/releases)
[![Status](https://img.shields.io/badge/Status-Public_Alpha-orange?style=for-the-badge&logo=shield)](FAQ.md#q3-is-this-production-ready)

---

## 🗺️ History Navigator

| Milestone / Version | Focus Area | Status |
| :--- | :--- | :--- |
| [**v0.3.2**](#032---2026-07-07) | Documentation & Integration Binary Cleanup | ✅ Released |
| [**v0.3.1**](#031---2026-06-17) | TEE Enclave Attestation & Shielded Swap Proxy | ✅ Released |
| [**v0.3.0**](#030---2026-06-08) | Shielded Vault Program & Modular CLI Refactoring | ✅ Released |
| [**v0.2.0**](#020---2026-05-18) | Core Refinement & Stability | ✅ Released |
| [**v0.1.0**](#010---2026-01-29) | Initial Public Release | ✅ Released |
| [**Milestone 1**](#milestone-1-research--evaluation-november-2025) | Research & Cryptography | ✅ Completed |
| [**Milestone 2**](#milestone-2-architecture--core-implementation-december-2025) | Core Engine Implementation | ✅ Completed |
| [**Milestone 3**](#milestone-3-integration--tooling-january-2026) | CLI, Node & Devnet Testing | ✅ Completed |
| [**Milestone 4**](#milestone-4-documentation--polish-late-january-2026) | Documentation & Branding | ✅ Completed |
| **Roadmap** | **What's coming next** | 🚀 [**View Full Roadmap →**](https://www.fhestate.org/roadmap) |

---

## [0.3.2] - 2026-07-07

**Documentation & Integration Binary Cleanup** — Publish-ready docs for Shielded Vault, TEE enclave flows, vault CLI helpers, and decentralized compute. Removes stale Cargo targets and renames the Devnet integration binary to feature-based naming.

### Added
* **`devnet_vault_enclave_flow` binary:** End-to-end Devnet script for attestation authority rotation, MRENCLAVE alignment, enclave registration, encrypted daily limits, transaction thresholds, and `shielded_swap_proxy` — registered in `Cargo.toml`.
* **Vault CLI documentation:** `docs/CLI.md` now documents `vault-transfer-hashes`, `vault-deposit-hash`, `vault-swap-hash`, `dao-tally-vote`, `store-ciphertext`, `decrypt-u32`, and `check-spending`.
* **Decentralized compute guide:** `docs/DECENTRALIZED-COMPUTE.md` linked from README — describes the `fhe-node` executor and five-layer compute stack.

### Changed
* **Renamed integration binary:** `devnet_phase5_flow` → `devnet_vault_enclave_flow` (feature-based naming; no internal phase labels).
* **Architecture docs:** Shielded Vault section retitled to *TEE Enclave Attestation & Shielded Swap*; removed misleading “Confidential Agent Network” framing from core Rust docs.
* **Version alignment:** `Cargo.toml` bumped to `0.3.2` to match README and changelog.
* **One-to-one documentation pass:** `SHIELDED-VAULT-PROGRAM.md`, `ARCHITECTURE.md`, `API.md`, `FHE_LOGIC.md`, and `FAQ.md` now document all 19 vault instructions, complete account layouts, vault CLI JSON schemas, Devnet program IDs, and integration binaries. Integration binaries aligned to live vault program ID `FuQzZCwPSRSVLT9gCgcft43a4RkapBJmSTC6CmdomeVQ`.

### Removed
* **Broken Cargo targets:** `threshold_decryption_simulator` and `fhe_ops_bench` (declared in `Cargo.toml` but no source files).

---

## [0.3.1] - 2026-06-17

**TEE Enclave Attestation & Shielded Swap Proxy** — Extends the Shielded Vault with Ed25519-gated enclave registration, shielded swap proxy execution, admin policy controls, and full TypeScript SDK parity verification on Solana Devnet.

### Added
* **TEE Enclave Attestation:** Implemented `register_enclave` instruction — requires a preceding `Ed25519SigVerify` precompile instruction signed by the current `attestation_authority` over the 64-byte payload `[enclave_pubkey (32) | mrenclave (32)]`. Prevents unauthorized enclave registration without valid on-chain attestation.
* **Shielded Swap Proxy:** Added `shielded_swap_proxy` instruction — executes a confidential swap authorized by a registered TEE enclave signer, updating the user's `EncryptedAccount.balance_hash` with the post-swap FHE commitment.
* **Admin Policy Controls:** Added `update_attestation_authority`, `update_approved_mrenclave`, `update_daily_limit` (256-byte FHE ciphertext), and `update_transaction_threshold` instructions for production key rotation and runtime policy management.
* **Documentation:** Added `docs/SHIELDED-VAULT-PROGRAM.md` — full on-chain reference including PDA layouts, instruction account tables, TEE attestation flow, and error codes.

### Verified — Devnet transactions (2026-06-17)

All instructions verified live on Solana Devnet (program `FuQzZCwPSRSVLT9gCgcft43a4RkapBJmSTC6CmdomeVQ`).

| Instruction | Signature |
|-------------|-----------|
| `update_attestation_authority` | [`RY77t39F...`](https://solscan.io/tx/RY77t39FVJbauHR1FvVYerNySWN4umdHzG1CrHKV7iSfZLqThkottBmk34EPXSzJkDqfRx7GHZBgvPnGXsYoLgj?cluster=devnet) |
| `update_approved_mrenclave` | [`3CVpwKf9...`](https://solscan.io/tx/3CVpwKf9Gwe7xGGX2USM8DDvFL46dFD5oZ7kVFZU8rLXvWx6BsiQKwvrkD3F3YfUxC1LU35qxTQPGxvP479ZpA2z?cluster=devnet) |
| `update_daily_limit` (256-byte FHE ciphertext) | [`gZVa4z5K...`](https://solscan.io/tx/gZVa4z5KjXj7ipmVAb7iq3ou6RCwk7rcWbkZ1mBYYUPwJmtyasWezQDgXFwYSuT7jSt19CdHWDcocXBuKCzxXFM?cluster=devnet) |
| `update_transaction_threshold` | [`2n5FXfbg...`](https://solscan.io/tx/2n5FXfbgwE1M9uPAD6LaUZcnACG1EdYKLtGYc61pCjegEE3P3g1tkj8KJtfu3dWKoZ1MKKsFHecuSdr1iTtLBFsM?cluster=devnet) |
| `register_enclave` (Ed25519 TEE attestation) | [`4NezbGtN...`](https://solscan.io/tx/4NezbGtN1wTHr4kPK184nrsSATG9ENYavEUvsJofgouYMWauemzsugM6Zhtoc6Fu7NzCK9q5QBNGi7E7dZtU4cEY?cluster=devnet) |
| `shielded_swap_proxy` (live registered enclave) | [`Lxw77MER...`](https://solscan.io/tx/Lxw77MERmAYbbneFhhPV8G2HMcoTxByvjHubGHdZvzbmtXMuXCvyVeMca7GKHpe3XchWpZ2LEK8S95YZG78E5Vg?cluster=devnet) |

> **On-chain FHE balance hash after swap:** `074a93885e30f3a82f4ab4969bad55ba5d187615256165988cf38d2247d8e9ca`

---

## [0.3.0] - 2026-06-08

**Shielded Vault Program & Modular CLI Refactoring** — Production readiness updates for decentralized confidential assets and key management flows.

### Added
* **Shielded Vault Program:** Added Anchor-based `programs/shielded_vault` implementing private balance pools with FHE transfers, SOL shielding, and FHE worker-authorized unshielding.
* **CLI Documentation:** Created `docs/CLI.md` mapping out setup and diagnostics tools.
* **CLI Features:** Added `doctor` (health checks), `status` (keys/cache overview), `balance`, `airdrop`, `keygen`, `history` (devnet transaction tracking), `watch` (wallet transaction polling), and automated `flow counter` commands.

### Changed
* **CLI Codebase:** Refactored a monolithic CLI structure into dedicated configuration (`config.rs`), cryptographic helpers (`crypto_util.rs`), RPC handlers (`rpc_util.rs`), output formatters (`output.rs`), and wallet utilities (`wallet.rs`).
* **Configuration:** Shifted CLI defaults to load from `.fhestate/config.json` and support `FHESTATE_*` environment overrides.
* **Cargo Configuration:** Cleaned up unused demo examples in `Cargo.toml`.

---

## [0.2.0] - 2026-05-18

**Phase 3 Completion: Developer SDK & CLI Tooling Release**

### Added
* **Modular CLI Subsystems:** Refactored the core `fhe-cli` code into dedicated clean helper modules (`config.rs`, `crypto_util.rs`, `output.rs`, `rpc_util.rs`, `wallet.rs`) for easier extensibility and high maintainability.
* **17 CLI Developer Commands:** Expanded the CLI from a simple 5-command prototype into a robust developer workspace:
  * `Demo` — Oone-sht key generation, local encryption, and memo transaction submit.
  * `Doctor` — Automated diagnostic testing for FHE keys, wallet configs, SOL balances, and RPC server latency.
  * `Status` — Compact summary of generated keys, active network mode, and cached ciphertexts.
  * `ConfigInit` — Secure generation of the default local configuration file.
  * `SubmitFile` — Submits any pre-encrypted `.bin` payload resolved from local cache.
  * `Encrypt` — Offline client-side encryption of u32 data and local content-addressable caching.
  * `Keygen` — Secure standalone lattice-based FHE key pair generation.
  * `Wallet` — Native Solana keypair generator.
  * `Balance` — Real-time wallet SOL balance check.
  * `Airdrop` — Direct command wrapper to request Devnet lamports.
  * `History` — Displays recent session transaction signatures and direct Solscan verification links.
  * `Cache` — List and inspect local content-addressed FHE ciphertexts.
  * `Watch` — Real-time on-chain transaction polling for the active wallet.
  * `Flow` — Initiates standard StateContainer PDA initialization and state update sequences.
* **Flexible Environment & Configuration overrides:** Standardized global options to load dynamically from `.fhestate/config.json` with native support for environment overrides (`FHESTATE_RPC`, `FHESTATE_PROGRAM_ID`, `FHESTATE_WALLET_PATH`).
* **CLI Reference Documentation:** Added a dedicated [docs/CLI.md](docs/CLI.md) reference detailing all subcommands and operation codes.
* **Developer Demos:** Added highly requested offline executable demos in `examples/` (`counter_demo.rs` and `voting_demo.rs` simulation scripts).
* **Versioning & Constants:** Added `PROTOCOL_VERSION` (v1) and `CRATE_VERSION` directly to `constants.rs` to maintain robust blockchain compatibility checks.
* **On-Chain Error Codegen:** Integrated specific numeric on-chain anchor errors (6000-6003 series for both Coordinator and Dark DAO) back into the core library for unified error decoding.

---

## [0.1.0] - 2026-01-29

**Initial public release of fhestate-rs** — Privacy-preserving computation on Solana using Fully Homomorphic Encryption.

### Development Timeline
This release represents **3 months of intensive research, development, and testing** (November 2025 - January 2026).

---

## 🏗️ Development History

### Milestone 1: Research & Evaluation (November 2025)
**Focus:** Evaluating FHE libraries and designing architecture.

#### 📅 November 10-15, 2025
* Researched multiple FHE libraries (SEAL, HElib, TFHE-rs, Concrete).
* Analyzed performance characteristics and Rust compatibility.
* Selected TFHE-rs v0.7.3 for production implementation.
* Initial benchmarking: encryption/decryption performance testing.

#### 📅 November 16-25, 2025
* Designed hybrid FHE-blockchain architecture.
* Evaluated Solana integration approaches (custom program vs SPL).
* Prototyped key generation and encryption flows.
* Documented cryptographic security requirements.

#### 📅 November 26-30, 2025
* Created initial project structure.
* Set up development environment and tooling.
* Defined API surface for SDK.
* Established testing strategy.

**Key Decisions:**
* ✅ **TFHE-rs** for FHE operations (best Rust support).
* ✅ **Hybrid model**: off-chain FHE computation + on-chain proofs.
* ✅ **SHA256** for cryptographic proof generation.
* ✅ **Solana Devnet** for initial deployment.

---

### Milestone 2: Architecture & Core Implementation (December 2025)
**Focus:** Building core infrastructure and FHE operations.

#### 📅 December 1-10, 2025
* Implemented core encryption/decryption module (`src/keys.rs`, `src/math.rs`).
* Built key management system with file-based storage.
* Created FHE operation wrappers (FheUint8 support).
* Initial integration tests for TFHE operations.

#### 📅 December 11-20, 2025
* Developed Solana integration layer.
* Implemented transaction signing and submission.
* Built proof generation system (SHA256 hashing).
* Created wallet management utilities.

#### 📅 December 21-31, 2025
* Designed coordinator program architecture (Anchor-based).
* Implemented task registry and state management.
* Built instruction handlers for task submission/completion.
* Created serialization/deserialization logic.

**Key Deliverables:**
* ✅ FHE encryption/decryption working.
* ✅ Homomorphic operations (add, sub, mul).
* ✅ Solana transaction submission.
* ✅ Cryptographic proof generation.

---

### Milestone 3: Integration & Tooling (January 2026)
**Focus:** Building CLI tools, executor node, and integration testing.

#### 📅 January 1-10, 2026
* Built `fhe-cli` command-line interface.
* Implemented task submission workflow.
* Created wallet creation and management commands.
* Integrated with Solana RPC client.

#### 📅 January 11-20, 2026
* Developed `fhe-node` background executor.
* Implemented blockchain listener for task detection.
* Built task queue and processing system.
* Created result submission logic.

#### 📅 January 21-25, 2026
* Comprehensive integration testing on Solana Devnet.
* Performance optimization and profiling.
* Bug fixes and error handling improvements.
* Security hardening (input validation, error messages).

**Key Features Added:**
* ✅ Complete CLI for task submission.
* ✅ Background node for task execution.
* ✅ Real Solana Devnet integration.
* ✅ Verified on-chain transactions.

---

### Milestone 4: Documentation & Polish (Late January 2026)
**Focus:** Production-ready release with comprehensive documentation.

#### 📅 January 26-27, 2026
* **Documentation Suite:**
  * Created comprehensive README.md.
  * Wrote QUICKSTART.md guide (5-minute setup).
  * Developed ARCHITECTURE.md (technical deep-dive).
  * Built API.md reference documentation.
  * Authored EXAMPLES.md with 12 code examples.
  * Wrote CONTRIBUTING.md guidelines.
  * Created FAQ.md for common questions.

#### 📅 January 28, 2026
* **Production Preparation:**
  * Removed all test/debug artifacts.
  * Added .gitignore for security (keys, wallets, logs).
  * Created MIT LICENSE.
  * Set up examples directory.
  * Final code cleanup and formatting.
  * Version 0.1.0 release preparation.

---

## 📝 [0.1.0] Release Notes

### Added

#### 🔐 Core FHE Operations
* Full TFHE-rs integration (v0.7.3).
* FheUint8 encryption/decryption.
* Homomorphic operations: addition, subtraction, multiplication.
* Client and Server key generation.
* Key serialization and file-based storage.

#### ⛓️ Solana Integration
* Transaction submission to Solana Devnet.
* SPL Memo program integration (demo).
* Custom Coordinator program (Anchor-based).
* Cryptographic proof generation (SHA256).
* Wallet management and signing.

#### 🛠️ Command-Line Tools
* `fhe_proof`: Key generation and local FHE demos.
* `fhe-cli`: Task submission to Solana blockchain.
* `fhe-node`: Background executor for FHE tasks.

#### 📖 Developer Experience
* Comprehensive documentation (7 detailed guides).
* 12 complete code examples.
* Quick start guide (5-minute setup).
* Architecture documentation & diagrams.
* API reference & Troubleshooting.

---

## 🔮 What's Next?

The full development roadmap — covering all 6 phases from Persistent Encrypted State through to Mainnet Hardening & Audit — is published at:

**👉 [fhestate.org/roadmap](https://www.fhestate.org/roadmap)**

---

## 🙏 Acknowledgments
* **Zama**: For the world-leading [TFHE-rs](https://github.com/zama-ai/tfhe-rs) library.
* **Solana**: For the high-performance blockchain infrastructure.
* **Rust Community**: For the exceptional tooling and ecosystem.

---

**Questions or Issues?** [Open an issue](https://github.com/fhestate/fhestate-rs/issues)
