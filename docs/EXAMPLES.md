# 🧪 FHESTATE Examples

**Practical code scenarios for building privacy-first applications.**

[![FHESTATE](https://img.shields.io/badge/FHESTATE-Examples-8A2BE2?style=for-the-badge&logo=rocket&logoColor=white)](https://github.com/fhestate/fhestate-rs)

---

## Scenario Navigator

*   **1. Fundamental Operations**
    *   [Basic Encrypt/Decrypt](#example-1-encrypt-and-decrypt) - The hello world of FHE.
    *   [Homomorphic Math](#example-2-homomorphic-addition) - Calculating on encrypted data.
    *   [Key Management](#example-3-generate-and-save-keys) - Storing and loading keys.

*   **2. Advanced Logic**
    *   [String Encryption](#example-5-encrypt-a-string) - Handling text data.
    *   [Shift Cipher](#example-6-homomorphic-shift-cipher) - Blind data transformations.
    *   [Solana Integration](#example-7-submit-to-solana) - Posting proofs to Devnet.

*   **3. Real-World Use Cases**
    *   [Private Voting](#example-8-private-voting) - Secret tallies.
    *   [Sealed-Bid Auction](#example-9-sealed-bid-auction) - Blind bidding logic.
    *   [Privacy-Preserving Mean](#example-10-privacy-preserving-mean) - Secure stats.

---

## 🏗️ Basic Examples

### Example 1: Encrypt and Decrypt
**Context**: This is the foundation. It demonstrates how to initialize the TFHE engine, generate a keypair, and perform the basic "Round Trip" (Plaintext -> Ciphertext -> Plaintext).

```rust
use tfhe::{generate_keys, ConfigBuilder, FheUint8};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Generate keys
    let config = ConfigBuilder::default().build();
    let (client_key, _server_key) = generate_keys(config);
    
    // 2. Encrypt a value
    let plaintext: u8 = 42;
    let ciphertext = FheUint8::encrypt(plaintext, &client_key);
    
    println!("Encrypted {} bytes", 
        bincode::serialize(&ciphertext)?.len());
    
    // 3. Decrypt the value
    let decrypted: u8 = ciphertext.decrypt(&client_key);
    
    // 4. Verify
    assert_eq!(plaintext, decrypted);
    println!("✅ Encryption/Decryption verified!");
    
    Ok(())
}
```

**Output:**
```
Encrypted 4096 bytes
✅ Encryption/Decryption verified!
```

---

### Example 2: Homomorphic Addition
**Context**: The "Magic" of FHE. This example shows two encrypted values being added together. The addition happens on the *ciphertexts* without ever revealing the underlying numbers to the processor.

```rust
use tfhe::{generate_keys, set_server_key, ConfigBuilder, FheUint8};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);
    
    // Encrypt two numbers
    let a = FheUint8::encrypt(15, &client_key);
    let b = FheUint8::encrypt(27, &client_key);
    
    // Add them (homomorphically)
    let sum = a + b;
    
    // Decrypt result
    let result: u8 = sum.decrypt(&client_key);
    
    assert_eq!(result, 42);
    println!("✅ 15 + 27 = {}", result);
    
    Ok(())
}
```

---

### Example 3: Generate and Save Keys
**Context**: In a real application, you don't generate keys on every run. This shows how to serialize the multi-megabyte FHE keys to binary files for persistent storage.

```rust
use tfhe::{generate_keys, ConfigBuilder};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate keys
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    
    // Create output directory
    fs::create_dir_all("./fhe_keys")?;
    
    // Serialize and save
    let client_bytes = bincode::serialize(&client_key)?;
    fs::write("./fhe_keys/client_key.bin", client_bytes)?;
    println!("✅ Client key saved");
    
    let server_bytes = bincode::serialize(&server_key)?;
    fs::write("./fhe_keys/server_key.bin", server_bytes)?;
    println!("✅ Server key saved");
    
    // Show file sizes
    let client_size = fs::metadata("./fhe_keys/client_key.bin")?.len();
    let server_size = fs::metadata("./fhe_keys/server_key.bin")?.len();
    
    println!("Client key: {} MB", client_size / 1_000_000);
    println!("Server key: {} MB", server_size / 1_000_000);
    
    Ok(())
}
```

---

### Example 4: Load Keys from Files
**Context**: Reconstructing the FHE state from disk. Essential for Nodes and Clients that need to resume operations.

```rust
use tfhe::{ClientKey, ServerKey, set_server_key};
use std::fs;

fn load_keys(
    client_path: &str,
    server_path: &str
) -> Result<(ClientKey, ServerKey), Box<dyn std::error::Error>> {
    // Load client key
    let client_bytes = fs::read(client_path)?;
    let client_key: ClientKey = bincode::deserialize(&client_bytes)?;
    
    // Load server key
    let server_bytes = fs::read(server_path)?;
    let server_key: ServerKey = bincode::deserialize(&server_bytes)?;
    
    Ok((client_key, server_key))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client_key, server_key) = load_keys(
        "./fhe_keys/client_key.bin",
        "./fhe_keys/server_key.bin"
    )?;
    
    set_server_key(server_key);
    
    println!("✅ Keys loaded successfully");
    
    // Use the keys
    let ct = FheUint8::encrypt(100, &client_key);
    let pt: u8 = ct.decrypt(&client_key);
    assert_eq!(pt, 100);
    
    Ok(())
}
```

---

## 🚀 Advanced Examples

### Example 5: Encrypt a String
**Context**: FHE primitives operate on bits/integers. To handle strings, we encrypt each byte individually. This example also shows how to generate SHA256 "Proofs" for each character.

```rust
use tfhe::{FheUint8, ClientKey, ConfigBuilder, generate_keys};
use sha2::{Sha256, Digest};

fn encrypt_string(text: &str, key: &ClientKey) -> Vec<FheUint8> {
    text.bytes()
        .map(|b| FheUint8::encrypt(b, key))
        .collect()
}

fn decrypt_string(ciphertexts: &[FheUint8], key: &ClientKey) -> String {
    let bytes: Vec<u8> = ciphertexts
        .iter()
        .map(|ct| ct.decrypt(key))
        .collect();
    
    String::from_utf8(bytes).unwrap()
}

fn hash_ciphertexts(ciphertexts: &[FheUint8]) -> Vec<String> {
    ciphertexts.iter().map(|ct| {
        let bytes = bincode::serialize(ct).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        format!("{:x}", hasher.finalize())
    }).collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigBuilder::default().build();
    let (client_key, _) = generate_keys(config);
    
    let message = "Hello FHE!";
    
    // Encrypt
    let encrypted = encrypt_string(message, &client_key);
    println!("Encrypted {} characters", encrypted.len());
    
    // Generate proofs
    let hashes = hash_ciphertexts(&encrypted);
    for (i, hash) in hashes.iter().enumerate() {
        println!("'{}' -> {}...", message.chars().nth(i).unwrap(), &hash[..16]);
    }
    
    // Decrypt
    let decrypted = decrypt_string(&encrypted, &client_key);
    assert_eq!(message, decrypted);
    println!("✅ Verified: {}", decrypted);
    
    Ok(())
}
```

---

### Example 6: Homomorphic Shift Cipher
**Context**: A standard Caesar shift, but performed purely on encrypted data. The "Server" (who does the shifting) knows a shift is happening but has no idea what the original letters are.

```rust
use tfhe::{FheUint8, set_server_key, ConfigBuilder, generate_keys, ClientKey};

fn shift_cipher(
    ciphertexts: &[FheUint8],
    shift: u8,
    client_key: &ClientKey
) -> Vec<FheUint8> {
    let shift_ct = FheUint8::encrypt(shift, client_key);
    
    ciphertexts.iter()
        .map(|ct| ct + &shift_ct)
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);
    
    let message = "ABC";
    
    // Encrypt
    let encrypted: Vec<FheUint8> = message.bytes()
        .map(|b| FheUint8::encrypt(b, &client_key))
        .collect();
    
    // Shift by 1 (homomorphically)
    let shifted = shift_cipher(&encrypted, 1, &client_key);
    
    // Decrypt
    let result: String = shifted.iter()
        .map(|ct| ct.decrypt(&client_key) as char)
        .collect();
    
    println!("Original: {}", message);
    println!("Shifted:  {}", result);
    assert_eq!(result, "BCD");
    
    Ok(())
}
```

---

### Example 7: Submit to Solana
**Context**: Taking the FHE proof on-chain. This uses the Solana RPC client to post the ciphertext hash to the SPL Memo program on Devnet.

```rust
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    instruction::{Instruction, AccountMeta},
    pubkey::Pubkey,
};
use std::str::FromStr;

fn submit_to_solana(
    proof_hash: [u8; 32],
    wallet_path: &str
) -> Result<String, Box<dyn std::error::Error>> {
    // Load wallet
    let wallet_file = std::fs::read_to_string(wallet_path)?;
    let wallet_bytes: Vec<u8> = serde_json::from_str(&wallet_file)?;
    let keypair = Keypair::from_bytes(&wallet_bytes)?;
    
    // Connect to Solana
    let rpc_client = RpcClient::new("https://api.devnet.solana.com".to_string());
    
    // Create instruction
    let memo_program = Pubkey::from_str(
        "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"
    )?;
    
    let memo_text = format!("FHE_PROOF:{:x}", 
        hex::encode(&proof_hash[..8]));
    
    let instruction = Instruction::new_with_bytes(
        memo_program,
        memo_text.as_bytes(),
        vec![AccountMeta::new(keypair.pubkey(), true)]
    );
    
    // Send transaction
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&keypair.pubkey()),
        &[&keypair],
        recent_blockhash
    );
    
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    
    Ok(signature.to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate a proof hash (example)
    let proof_hash = [0u8; 32]; // In real use, hash of ciphertext
    
    // Submit
    let tx_sig = submit_to_solana(proof_hash, "./deploy-wallet.json")?;
    
    println!("✅ Transaction submitted!");
    println!("Signature: {}", tx_sig);
    
    Ok(())
}
```

---

## 🎯 Use Case Examples

### Example 8: Private Voting
**Context**: Implement a voting system where individual choices are encrypted. The "Tally" is computed homomorphically, so the final result is revealed while individual votes remain secret forever.

```rust
use tfhe::{FheUint8, ClientKey, ServerKey, set_server_key, ConfigBuilder, generate_keys};

struct VotingSystem {
    client_key: ClientKey,
    server_key: ServerKey,
}

impl VotingSystem {
    fn new() -> Self {
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);
        set_server_key(server_key.clone());
        
        Self { client_key, server_key }
    }
    
    fn cast_vote(&self, vote: u8) -> FheUint8 {
        FheUint8::encrypt(vote, &self.client_key)
    }
    
    fn tally_votes(&self, encrypted_votes: Vec<FheUint8>) -> FheUint8 {
        encrypted_votes.into_iter()
            .fold(FheUint8::encrypt(0, &self.client_key), |acc, vote| acc + vote)
    }
    
    fn reveal_count(&self, encrypted_count: &FheUint8) -> u8 {
        encrypted_count.decrypt(&self.client_key)
    }
}

fn main() {
    let system = VotingSystem::new();
    let votes = vec![system.cast_vote(1), system.cast_vote(0), system.cast_vote(1)];
    let total = system.tally_votes(votes);
    println!("✅ Results revealed: {}", system.reveal_count(&total));
}
```

---

### Example 9: Sealed-Bid Auction
**Context**: Find the highest bid without revealing any bidding history. This uses homomorphic comparisons to determine the winner blindly.

```rust
use tfhe::{FheUint8, ClientKey, set_server_key, ConfigBuilder, generate_keys};

struct Auction {
    client_key: ClientKey,
}

impl Auction {
    fn new() -> Self {
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);
        set_server_key(server_key);
        Self { client_key }
    }
    
    fn submit_bid(&self, amount: u8) -> FheUint8 {
        FheUint8::encrypt(amount, &self.client_key)
    }
    
    fn reveal_winner(&self, winning_bid: &FheUint8) -> u8 {
        winning_bid.decrypt(&self.client_key)
    }
}
```

---

### Example 10: Privacy-Preserving Mean
**Context**: Calculate the average of a sensitive dataset (e.g., salaries or medical data) without any single data point being exposed.

```rust
fn encrypted_mean(values: &[FheUint8], client_key: &ClientKey) -> u8 {
    let sum = values.iter()
        .fold(FheUint8::encrypt(0, client_key), |acc, v| acc + v);
    
    let count = values.len() as u8;
    sum.decrypt(client_key) / count
}
```

---
### Example 11: Using StateTransition (Node-Side)
**Context**: This is what `fhe-node` does internally for every task. `StateTransition::apply()` is the core of the off-chain computation engine — it loads the old state, runs the FHE op, stores the new state, and returns the SHA256 proof hash to post on-chain.

```rust
use fhestate_rs::{KeyManager, FheMath, LocalCache, StateTransition};
use fhestate_rs::constants::ops;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load keys and activate server key (required before any FHE op)
    let keys = KeyManager::load("./fhe_keys")?;
    keys.activate(); // sets ServerKey in thread-local storage

    let cache = LocalCache::new(".fhe_cache");

    // --- CLIENT SIDE: Encrypt input ---
    let ct_input = FheMath::encrypt_u32(100, &keys.client_key);
    let input_bytes = FheMath::serialize_u32(&ct_input)?;

    // --- NODE SIDE: Apply FHE operation ---
    // First call: no existing state — input becomes initial state
    let (uri_v1, hash_v1) = StateTransition::apply(
        &cache,
        None,         // No prior state (fresh account)
        &input_bytes,
        ops::ADD,
    )?;
    println!("Initial state URI: {}", uri_v1);
    println!("Post to chain → state_hash: {}", hex::encode(hash_v1));

    // Second call: add 200 to existing state
    let ct_second = FheMath::encrypt_u32(200, &keys.client_key);
    let second_bytes = FheMath::serialize_u32(&ct_second)?;

    let (uri_v2, hash_v2) = StateTransition::apply(
        &cache,
        Some(&uri_v1), // Current state_uri from StateContainer PDA
        &second_bytes,
        ops::ADD,
    )?;
    println!("New state URI: {}", uri_v2);

    // --- CLIENT SIDE: Verify hash and decrypt ---
    let result_bytes = cache.load(&uri_v2)?;
    assert_eq!(FheMath::hash(&result_bytes), hash_v2, "Hash mismatch!");

    let result = FheMath::decrypt_u32(
        &FheMath::deserialize_u32(&result_bytes)?,
        &keys.client_key
    );
    println!("✅ Result: {} (expected 300)", result);
    assert_eq!(result, 300);
    Ok(())
}
```

---

### Example 12: Content-Addressed Cache Operations
**Context**: `LocalCache` stores ciphertexts by SHA256 hash of their content. Same bytes → same URI, always. Useful for deduplication and integrity checking.

```rust
use fhestate_rs::LocalCache;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cache = LocalCache::new(".fhe_cache");
    let data = b"example ciphertext bytes";

    // Store returns content-addressed URI (local://<64-char-sha256>)
    let uri = cache.store(data)?;
    println!("Stored at: {}", uri);

    // Deterministic — same data always yields same URI
    assert_eq!(cache.store(data)?, uri);

    // Load by URI
    assert_eq!(cache.load(&uri)?, data);

    // Resolve handles both local:// and ipfs:// schemes
    let _ = cache.resolve(&uri)?;

    // Introspection
    println!("Cache entries: {}", cache.list()?.len());
    println!("Cache size: {} KB", cache.size()? / 1024);

    // Cleanup
    cache.delete(&uri)?;
    println!("✅ Done");
    Ok(())
}
```
