# 📜 FHESTATE CHANGELOG

**The chronological evolution of confidential computing on Solana.**

[![Version](https://img.shields.io/badge/Version-v0.3.0-8A2BE2?style=for-the-badge&logo=rocket&logoColor=white)](https://github.com/fhestate/fhestate-rs/releases)
[![Status](https://img.shields.io/badge/Status-Public_Alpha-orange?style=for-the-badge&logo=shield)](FAQ.md#q3-is-this-production-ready)

---

## 🗺️ History Navigator

| Milestone / Version | Focus Area | Status |
| :--- | :--- | :--- |
| [**v0.2.0 (Current)**](#020---2026-04-17) | Core Refinement & Stability | ✅ Released |
| [**v0.1.0**](#010---2026-01-29) | Initial Public Release | ✅ Released |
| [**Milestone 1**](#milestone-1-research--evaluation-november-2025) | Research & Cryptography | ✅ Completed |
| [**Milestone 2**](#milestone-2-architecture--core-implementation-december-2025) | Core Engine Implementation | ✅ Completed |
| [**Milestone 3**](#milestone-3-integration--tooling-january-2026) | CLI, Node & Devnet Testing | ✅ Completed |
| [**Milestone 4**](#milestone-4-documentation--polish-late-january-2026) | Documentation & Branding | ✅ Completed |
| **Roadmap** | **What's coming next** | 🚀 [**View Full Roadmap →**](https://www.fhestate.org/roadmap) |

---

## [0.3.0] - 2026-06-08

**Shielded Vault Program & Modular CLI Refactoring** — Production readiness updates for decentralized confidential assets and key management flows.

### Added
* **Shielded Vault Program:** Added Anchor-based `programs/shielded_vault` implementing private balance pools with FHE transfers, SOL shielding, and FHE worker-authorized unshielding.
* **CLI Documentation:** Created `docs/CLI.md` mapping out setup and diagnostics tools.
* **CLI Features:** Added `doctor` (health checks), `status` (keys/cache overview), `balance`, `airdrop`, `keygen`, `history` (devnet transaction tracking), `watch` (wallet transaction polling), and automated `flow counter` commands.

### Changed
* **CLI Codebase:** Refactored a monolithic CLI structure into dedicated configuration (`config.rs`), cryptographic helpers (`crypto_util.rs`), RPC handlers (`rpc_util.rs`), output formatters (`output.rs`), and wallet utilities (`wallet.rs`).
* **Configuration:** Shifted CLI defaults to load from `.fhestate/config.json` and support FHESTATE_* environment overrides.
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
