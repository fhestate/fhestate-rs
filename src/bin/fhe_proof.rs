use tfhe::prelude::*;
use tfhe::{generate_keys, ConfigBuilder, FheUint8, set_server_key, ClientKey, ServerKey};
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::{Read, Write};
use std::error::Error;
use clap::{Parser, Subcommand};
use log::info;
use std::path::Path;

/// FHEstate Verification Tool (CLI)
/// 
/// Client-side tool to generate keys, submit FHE tasks, and verify proofs.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a fresh FHE Keypair (Client + Server Keys)
    Keygen {
        /// Output directory for keys
        #[arg(short, long, default_value = "fhe_keys")]
        out_dir: String,
    },
    /// Run the "FHestate is coming" End-to-End Demo
    Demo {
        /// RPC URL
        #[arg(long, default_value = "https://api.devnet.solana.com")]
        rpc_url: String,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let args = Args::parse();

    match args.command {
        Commands::Keygen { out_dir } => run_keygen(&out_dir),
        Commands::Demo { rpc_url } => run_demo(&rpc_url),
    }
}

/// Command: Generate Keys
fn run_keygen(out_dir: &str) -> Result<(), Box<dyn Error>> {
    info!("Generating fully homomorphic encryption keys...");
    
    // Create dir if not exists
    std::fs::create_dir_all(out_dir)?;

    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);

    // Save Client Key
    let client_path = format!("{}/client_key.bin", out_dir);
    let mut file = File::create(&client_path)?;
    bincode::serialize_into(&mut file, &client_key)?;
    info!("Saved Client Key to: {}", client_path);

    // Save Server Key
    info!("Saving Server Key (this is large ~100MB, please wait)...");
    let server_path = format!("{}/server_key.bin", out_dir);
    let file = File::create(&server_path)?;
    let mut writer = std::io::BufWriter::new(file);
    bincode::serialize_into(&mut writer, &server_key)?;
    info!("Saved Server Key to: {}", server_path);
    
    info!("✅ Key Generation Complete.");
    Ok(())
}

/// Command: Run Demo
fn run_demo(_rpc_url: &str) -> Result<(), Box<dyn Error>> {
    println!("═══════════════════════════════════════════════════════════");
    println!("     FHE STATE: PRODUCTION DEMO (Monorepo-Free)");
    println!("     Target: 'FHestate is coming'");
    println!("═══════════════════════════════════════════════════════════\n");

    // 1. Loading Keys
    info!("Loading FHE Keys from 'fhe_keys/'...");
    if !Path::new("fhe_keys/client_key.bin").exists() {
        return Err("Keys not found. Run 'cargo run --bin fhe_proof -- keygen' first.".into());
    }

    let mut file = File::open("fhe_keys/client_key.bin")?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let client_key: ClientKey = bincode::deserialize(&bytes)?;

    let mut file = File::open("fhe_keys/server_key.bin")?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let server_key: ServerKey = bincode::deserialize(&bytes)?;
    
    set_server_key(server_key);
    info!("Keys Loaded & Activated.");

    // 2. Encryption
    let target_sentence = "SKD is ready";
    info!("Encrypting Sentence: '{}'", target_sentence);
    
    let mut ciphertexts = Vec::new();
    println!("\n--- Ciphertext Hashes ---");
    for b in target_sentence.as_bytes() {
        let ct = FheUint8::encrypt(*b, &client_key);
        // Serialize to get bytes for hashing
        let ct_bytes = bincode::serialize(&ct)?;
        let mut hasher = Sha256::new();
        hasher.update(&ct_bytes);
        let hash = hasher.finalize();
        println!("'{}' -> {:x}", *b as char, hash);
        
        ciphertexts.push(ct);
        std::io::stdout().flush()?;
    }
    println!("-------------------------\n");
    info!("Encryption Complete. {} Characters secured.", ciphertexts.len());

    // 3. Homomorphic Computation (Shift + 1)
    info!("Executing Homomorphic Shift (+1) on encrypted data...");
    let one = FheUint8::encrypt(1u8, &client_key);
    
    let mut result_ciphertexts = Vec::new();
    for ct in &ciphertexts {
        let res = ct + &one;
        result_ciphertexts.push(res);
        print!(".");
        std::io::stdout().flush()?;
    }
    println!(""); 
    info!("Computation Complete.");

    // 4. Decryption & Verify
    info!("Decrypting Result for Verification...");
    let mut decrypted_string = String::new();
    
    for ct in result_ciphertexts {
        let val: u8 = ct.decrypt(&client_key);
        decrypted_string.push(val as char);
    }

    println!("   Original:  {}", target_sentence);
    println!("   Decrypted: {}", decrypted_string);
    
    let unshifted: String = decrypted_string.chars().map(|c| (c as u8 - 1) as char).collect();
    
    if unshifted == target_sentence {
         println!("\n   STATUS: ✅ VERIFIED SUCCESS");
    } else {
         println!("\n   STATUS: ✗ FAILED");
    }

    Ok(())
}
