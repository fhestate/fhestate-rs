# ⚡ FHESTATE Quick Start

**Get up and running with FHESTATE in 5 minutes.**

[![FHESTATE](https://img.shields.io/badge/FHESTATE-Quickstart-8A2BE2?style=for-the-badge&logo=rocket&logoColor=white)](https://github.com/fhestate/fhestate-rs)

---

## 🗺️ Step-by-Step Navigator

*   **1. Prerequisites** - Environment setup for Rust and Solana.
*   **2. Installation** - Cloning and building the production binaries.
*   **3. Configuration** - Wallet setup and Devnet funding.
*   **4. Your First Transaction** - Keygen, local demo, and on-chain submission.
*   **5. Advanced Setup** - Program deployment (Default vs. Custom).
*   **6. Operations & Performance** - Troubleshooting and speed tips.

---

## 🛠️ 1. Prerequisites

Before you begin, ensure you have the following installed:

*   **Rust (1.70 or higher)**
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    echo 'source $HOME/.cargo/env' >> ~/.profile
    source $HOME/.cargo/env
    ```

*   **Solana CLI (1.18 or higher)**
    ```bash
    sh -c "$(curl -sSfL https://raw.githubusercontent.com/solana-labs/solana/master/install/solana-install-init.sh)" -- v1.18.26
    
    # Add to path (Codespaces)
    export PATH="/home/codespace/.local/share/solana/install/active_release/bin:$PATH"
    ```

*   **Git**
    ```bash
    # Ubuntu/Debian
    sudo apt install git
    
    # macOS
    brew install git
    ```

---

## ⚙️ 2. Installation

### Step 1: Clone the Repository
```bash
git clone https://github.com/FHESTATE/FHESTATE-rs.git
cd FHESTATE-rs
```

### Step 2: Build the Project
```bash
# Build all binaries in release mode
cargo build --release

# This will take 5-10 minutes on first build
```

---

## ⛓️ 3. Configuration

### Step 3: Configure Solana
```bash
# Configure to Devnet
solana config set --url https://api.devnet.solana.com

# Create a new wallet
solana-keygen new --outfile deploy-wallet.json --no-bip39-passphrase

# Fund the wallet (Important: Specify the keypair!)
solana airdrop 2 -k deploy-wallet.json
```

---

## 🚀 4. Your First FHE Transaction

### Step 4: Generate FHE Keys
```bash
# Run in release mode for speed
cargo run --release --bin fhe_proof -- keygen --out-dir fhe_keys
```

**Expected Output:**
```text
[INFO] Generating fully homomorphic encryption keys...
[INFO] Saved Client Key to: fhe_keys/client_key.bin
[INFO] Saved Server Key to: fhe_keys/server_key.bin
```

> [!NOTE]
> *   `client_key.bin`: Your private key for encryption/decryption. **Keep this secret!**
> *   `server_key.bin`: Public key used by the node for math operations.

---

### Step 5: Run Local FHE Demo
Verify the blind computation logic on your machine.
```bash
cargo run --release --bin fhe_proof -- demo
```

**Expected Output:**
```text
═══════════════════════════════════════════════════════════
     FHE STATE: PRODUCTION DEMO
     Target: 'Solana Privacy Ops'
═══════════════════════════════════════════════════════════
--- Ciphertext Hashes ---
'S' -> cc9f8376ad33bc...
...
   Original:  Solana Privacy Ops
   Decrypted: Tpmbob!Qsjwbdz!Pqt (Shifted by 1)
   STATUS: ✅ VERIFIED SUCCESS
```

> [!TIP]
> **Why do I see a pattern?** (S -> T, K -> L)
> The demo is programmed to perform a **homomorphic +1 shift** to prove mathematical accuracy. Only you (the key holder) can see this pattern after decryption. To the Solana Node and any external attacker, the data remains 128-bit secure random noise.

---

### Step 6: Submit to Solana
Post a cryptographic proof to the blockchain.
```bash
cargo run --release --bin fhe-cli -- submit --op 1
```

**Expected Output:**
```text
[INFO] Submitting FHE Task to Solana
[INFO]    Success! Transaction Hash: 4w9MES...
```

---

## 🔧 5. Program Deployment Options

### Option A: Use Default Program ✅ (Recommended)
The SDK uses a pre-deployed program by default - **no additional steps required!**
*   **Program ID**: `MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr` (SPL Memo)
*   **Best for**: Prototyping, testing, and learning FHE concepts.

### Option B: Deploy Your Own Program 🔒
For production or custom cryptographic requirements:

1.  **Build the Program**:
    ```bash
    cd programs/coordinator
    cargo build-bpf
    ```
2.  **Deploy to Devnet**:
    ```bash
    solana program deploy target/deploy/coordinator.so
    ```
3.  **Use Your ID**:
    ```bash
    cargo run --bin fhe-cli -- submit --program YOUR_PROGRAM_ID --op 1
    ```

---

## 🛠️ 6. Operations & Performance

### Troubleshooting Matrix
| Error | Cause | Solution |
| :--- | :--- | :--- |
| **"Keys not found"** | Missing binary files | Run the `keygen` command in Step 4. |
| **"Insufficient funds"** | No Devnet SOL | Run `solana airdrop 2 -k deploy-wallet.json`. |
| **"RPC connection failed"** | Network timeout | Check `solana config get`. |

### Performance Tips
*   **ALWAYS use `--release`**: Debug mode for FHE is 50x slower.
*   **GPU Acceleration**: FHESTATE is migrating to `tfhe-cuda` for 10x faster multiplication in Q3.
*   **Parallel Builds**: Use `cargo build --release -j$(nproc)` for faster compilation.

---

## 🗺️ Next Steps

1.  **Read the Architecture**: [ARCHITECTURE.md](ARCHITECTURE.md)
2.  **Explore the API**: [API.md](API.md)
3.  **Check the FAQ**: [FAQ.md](FAQ.md)

---