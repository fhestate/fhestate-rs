use crate::config::{
    is_memo_mode, registry_mode, save_config, CliConfig, ConfigOverrides, CONFIG_FILE, REGISTRY_FILE,
};
use crate::crypto_util::{encrypt_u32, ensure_fhe_keys, sha256_hex};
use crate::output::{self, fail, kv, line, ok, title, tx_success, warn};
use crate::rpc_util::{get_balance_sol, get_signatures, request_airdrop, rpc_slot};
use crate::wallet::{generate_wallet, load_keypair};
use fhestate_rs::constants::CRATE_VERSION;
use fhestate_rs::{KeyManager, LocalCache};
use sha2::{Digest, Sha256};
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

fn submit_memo_with_uri(cfg: &CliConfig, uri: &str) -> Result<(), Box<dyn Error>> {
    let prog_id = Pubkey::from_str(&cfg.program_id)?;
    let payer = load_keypair(&cfg.wallet_path)?;
    let rpc = RpcClient::new(cfg.rpc_url.clone());
    let ix = Instruction::new_with_bytes(
        prog_id,
        uri.as_bytes(),
        vec![AccountMeta::new_readonly(payer.pubkey(), true)],
    );
    line("Sending SPL Memo transaction...");
    let signature = rpc.send_and_confirm_transaction(&Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        rpc.get_latest_blockhash()?,
    ))?;
    tx_success(&signature.to_string());
    Ok(())
}

// ─── Legacy / core on-chain commands (refactored) ───────────────────────────

pub fn setup(cfg: &CliConfig) -> Result<(), Box<dyn Error>> {
    let prog_id = Pubkey::from_str(&cfg.program_id)?;
    let is_memo = is_memo_mode(&cfg.program_id);

    title("FHESTATE Setup");
    ensure_fhe_keys(&cfg.key_dir)?;

    if is_memo {
        ok("Memo demo mode — no coordinator deploy needed");
        let mut config_file = File::create(REGISTRY_FILE)?;
        write!(config_file, "MEMO_MODE")?;
        save_config(cfg)?;
        line("Saved .fhestate/config.json");
        return Ok(());
    }

    let payer = load_keypair(&cfg.wallet_path)?;
    let rpc = RpcClient::new(cfg.rpc_url.clone());

    let registry_keypair = Keypair::new();
    let registry_pubkey = registry_keypair.pubkey();

    let mut disc_hasher = Sha256::new();
    disc_hasher.update(b"global:initialize");
    let disc = disc_hasher.finalize();
    let mut data = disc[..8].to_vec();
    data.extend_from_slice(&100_000_000u64.to_le_bytes());

    let ix_reg = Instruction::new_with_bytes(
        prog_id,
        &data,
        vec![
            AccountMeta::new(registry_pubkey, true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
    );

    let (state_pda, _) =
        Pubkey::find_program_address(&[b"state", payer.pubkey().as_ref()], &prog_id);

    let mut state_disc_hasher = Sha256::new();
    state_disc_hasher.update(b"global:initialize_state");
    let state_disc = state_disc_hasher.finalize();

    let ix_state = Instruction::new_with_bytes(
        prog_id,
        &state_disc[..8],
        vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
    );

    line("Sending coordinator setup transaction...");
    let signature = rpc.send_and_confirm_transaction(&Transaction::new_signed_with_payer(
        &[ix_reg, ix_state],
        Some(&payer.pubkey()),
        &[&payer, &registry_keypair],
        rpc.get_latest_blockhash()?,
    ))?;

    let mut config_file = File::create(REGISTRY_FILE)?;
    write!(config_file, "{registry_pubkey}")?;
    save_config(cfg)?;
    tx_success(&signature.to_string());
    kv("Registry", &registry_pubkey.to_string());
    Ok(())
}

pub fn submit_task(
    cfg: &CliConfig,
    op: u8,
    value: u32,
    target_owner: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    title("Submit FHE Task");
    let prog_id = Pubkey::from_str(&cfg.program_id)?;
    let is_memo = is_memo_mode(&cfg.program_id);

    let registry_pubkey = if !is_memo {
        let registry_addr_str = fs::read_to_string(REGISTRY_FILE)
            .map_err(|_| "Registry not found. Run: fhe-cli setup")?;
        if registry_addr_str.trim() == "MEMO_MODE" {
            return Err(
                "CLI is in memo mode. Run setup with --program <COORDINATOR_ID> for coordinator."
                    .into(),
            );
        }
        Some(Pubkey::from_str(registry_addr_str.trim())?)
    } else {
        None
    };

    let payer = load_keypair(&cfg.wallet_path)?;
    let rpc = RpcClient::new(cfg.rpc_url.clone());

    let ciphertext_bytes = encrypt_u32(value, &cfg.key_dir)?;
    let commitment = sha256_hex(&ciphertext_bytes);
    kv("Plain value", &value.to_string());
    kv("Ciphertext bytes", &ciphertext_bytes.len().to_string());
    kv("SHA-256 commitment", &commitment);

    let cache = LocalCache::new(&cfg.cache_dir);
    let uri = cache.store(&ciphertext_bytes)?;
    kv("Cache URI", &uri);

    let task_keypair_opt = if is_memo { None } else { Some(Keypair::new()) };

    if is_memo {
        line("Mode: SPL Memo (devnet demo)");
        return submit_memo_with_uri(cfg, &uri);
    }

    let ix = {
        line("Mode: Coordinator");
        let mut hasher = Sha256::new();
        hasher.update(&ciphertext_bytes);
        let input_hash: [u8; 32] = hasher.finalize().into();

        let mut disc_hasher = Sha256::new();
        disc_hasher.update(b"global:submit_task");
        let disc = disc_hasher.finalize();

        let mut data = disc[..8].to_vec();
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        data.extend_from_slice(&id.to_le_bytes());
        data.extend_from_slice(&input_hash);
        let uri_bytes = uri.as_bytes();
        data.extend_from_slice(&(uri_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(uri_bytes);
        data.push(op);

        if let Some(target_str) = target_owner {
            let target_pk = Pubkey::from_str(target_str)?;
            data.push(1);
            data.extend_from_slice(target_pk.as_ref());
        } else {
            data.push(0);
        }

        Instruction::new_with_bytes(
            prog_id,
            &data,
            vec![
                AccountMeta::new(registry_pubkey.unwrap(), false),
                AccountMeta::new(task_keypair_opt.as_ref().unwrap().pubkey(), true),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            ],
        )
    };

    line("Sending coordinator transaction...");
    let tkp = task_keypair_opt.as_ref().unwrap();
    let signature = rpc.send_and_confirm_transaction(&Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer, tkp],
        rpc.get_latest_blockhash()?,
    ))?;

    tx_success(&signature.to_string());
    Ok(())
}

pub fn submit_input(
    cfg: &CliConfig,
    operation: u8,
    value: u32,
    target_owner: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    title("Submit Inline Input");
    let rpc = RpcClient::new(cfg.rpc_url.clone());
    let prog_id = Pubkey::from_str(&cfg.program_id)?;
    let payer = load_keypair(&cfg.wallet_path)?;

    let encrypted_data = encrypt_u32(value, &cfg.key_dir)?;
    if encrypted_data.len() > 1000 {
        warn(&format!(
            "Ciphertext is {} bytes — may exceed Solana tx limit. Prefer: fhe-cli submit",
            encrypted_data.len()
        ));
    }

    let cache = LocalCache::new(&cfg.cache_dir);
    let uri = cache.store(&encrypted_data)?;
    kv("Cache URI", &uri);

    let mut disc_hasher = Sha256::new();
    disc_hasher.update(b"global:submit_input");
    let disc = disc_hasher.finalize();

    let mut data = disc[..8].to_vec();
    data.extend_from_slice(&(encrypted_data.len() as u32).to_le_bytes());
    data.extend_from_slice(&encrypted_data);
    data.push(operation);

    if let Some(target_str) = target_owner {
        let target_pk = Pubkey::from_str(target_str)?;
        data.push(1);
        data.extend_from_slice(target_pk.as_ref());
    } else {
        data.push(0);
    }

    let target_pk = if let Some(target) = target_owner {
        Pubkey::from_str(target)?
    } else {
        payer.pubkey()
    };

    let (state_pda, _) =
        Pubkey::find_program_address(&[b"state", target_pk.as_ref()], &prog_id);

    let ix = Instruction::new_with_bytes(
        prog_id,
        &data,
        vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new(payer.pubkey(), true),
        ],
    );

    let signature = rpc.send_and_confirm_transaction(&Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        rpc.get_latest_blockhash()?,
    ))?;

    tx_success(&signature.to_string());
    Ok(())
}

pub fn init_state(cfg: &CliConfig) -> Result<(), Box<dyn Error>> {
    title("Initialize State PDA");
    let payer = load_keypair(&cfg.wallet_path)?;
    let prog_id = Pubkey::from_str(&cfg.program_id)?;
    let rpc = RpcClient::new(cfg.rpc_url.clone());

    let (state_pda, _) =
        Pubkey::find_program_address(&[b"state", payer.pubkey().as_ref()], &prog_id);
    kv("State PDA", &state_pda.to_string());

    let mut disc_hasher = Sha256::new();
    disc_hasher.update(b"global:initialize_state");
    let disc = disc_hasher.finalize();

    let ix = Instruction::new_with_bytes(
        prog_id,
        &disc[..8],
        vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
    );

    let signature = rpc.send_and_confirm_transaction(&Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        rpc.get_latest_blockhash()?,
    ))?;

    tx_success(&signature.to_string());
    Ok(())
}

pub fn reveal_task(cfg: &CliConfig, task_pubkey: &str) -> Result<(), Box<dyn Error>> {
    title("Request Reveal");
    let rpc = RpcClient::new(cfg.rpc_url.clone());
    let prog_id = Pubkey::from_str(&cfg.program_id)?;
    let task_pk = Pubkey::from_str(task_pubkey)?;
    let payer = load_keypair(&cfg.wallet_path)?;

    let mut disc_hasher = Sha256::new();
    disc_hasher.update(b"global:request_reveal");
    let disc = disc_hasher.finalize();

    let ix = Instruction::new_with_bytes(
        prog_id,
        &disc[..8],
        vec![
            AccountMeta::new(task_pk, false),
            AccountMeta::new(payer.pubkey(), true),
        ],
    );

    let signature = rpc.send_and_confirm_transaction(&Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        rpc.get_latest_blockhash()?,
    ))?;

    tx_success(&signature.to_string());
    Ok(())
}

// ─── New CLI commands ───────────────────────────────────────────────────────

pub fn status(cfg: &CliConfig) -> Result<(), Box<dyn Error>> {
    title("FHESTATE CLI Status");
    kv("CLI version", CRATE_VERSION);
    kv("RPC", &cfg.rpc_url);
    kv("Program", &cfg.program_id);
    kv("Wallet file", &cfg.wallet_path);
    kv("FHE keys", &cfg.key_dir);
    kv("Cache", &cfg.cache_dir);

    let keys_ok = fhestate_rs::keys::keys_exist(&cfg.key_dir);
    if keys_ok {
        ok("FHE keys present");
    } else {
        warn("FHE keys missing — run: fhe-cli setup or fhe-cli keygen");
    }

    let mode = registry_mode().unwrap_or_else(|_| "unknown".to_string());
    kv("Registry mode", &mode);

    if Path::new(&cfg.wallet_path).exists() {
        let kp = load_keypair(&cfg.wallet_path)?;
        kv("Wallet address", &kp.pubkey().to_string());
        let rpc = RpcClient::new(cfg.rpc_url.clone());
        match get_balance_sol(&rpc, &kp.pubkey()) {
            Ok(sol) => kv("Balance", &format!("{sol:.4} SOL")),
            Err(e) => warn(&format!("Could not read balance: {e}")),
        }
    } else {
        warn(&format!("Wallet file missing: {}", cfg.wallet_path));
    }

    let cache = LocalCache::new(&cfg.cache_dir);
    if let Ok(uris) = cache.list() {
        kv("Cached ciphertexts", &uris.len().to_string());
    }

    Ok(())
}

pub fn doctor(cfg: &CliConfig) -> Result<(), Box<dyn Error>> {
    title("FHESTATE Doctor");
    let mut issues = 0u32;

    if fhestate_rs::keys::keys_exist(&cfg.key_dir) {
        ok("FHE keys");
    } else {
        fail("FHE keys missing");
        line("Fix: fhe-cli keygen");
        issues += 1;
    }

    if Path::new(&cfg.wallet_path).exists() {
        match load_keypair(&cfg.wallet_path) {
            Ok(kp) => {
                ok("Wallet file");
                kv("Address", &kp.pubkey().to_string());
            }
            Err(e) => {
                fail(&format!("Wallet invalid: {e}"));
                issues += 1;
            }
        }
    } else {
        fail("Wallet file missing");
        line(&format!("Fix: fhe-cli wallet new --out {}", cfg.wallet_path));
        issues += 1;
    }

    let rpc = RpcClient::new(cfg.rpc_url.clone());
    match rpc_slot(&rpc) {
        Ok(slot) => {
            ok("RPC reachable");
            kv("Slot", &slot.to_string());
        }
        Err(e) => {
            fail(&format!("RPC error: {e}"));
            issues += 1;
        }
    }

    if Path::new(&cfg.wallet_path).exists() {
        if let Ok(kp) = load_keypair(&cfg.wallet_path) {
            match get_balance_sol(&rpc, &kp.pubkey()) {
                Ok(sol) if sol >= 0.01 => ok(&format!("Balance OK ({sol:.4} SOL)")),
                Ok(sol) => {
                    warn(&format!("Low balance: {sol:.4} SOL"));
                    line("Fix: fhe-cli airdrop  (or https://faucet.solana.com)");
                    issues += 1;
                }
                Err(e) => {
                    warn(&format!("Balance check failed: {e}"));
                    issues += 1;
                }
            }
        }
    }

    println!();
    if issues == 0 {
        ok("All checks passed — ready for fhe-cli demo");
    } else {
        warn(&format!("{issues} issue(s) found — fix before submitting txs"));
    }
    Ok(())
}

pub fn demo(cfg: &CliConfig, value: u32) -> Result<(), Box<dyn Error>> {
    title("FHESTATE Devnet Demo");
    line("This will: check keys → check wallet → encrypt → submit SPL Memo tx");
    println!();

    ensure_fhe_keys(&cfg.key_dir)?;

    if !Path::new(&cfg.wallet_path).exists() {
        warn("Creating new wallet...");
        let kp = generate_wallet(&cfg.wallet_path)?;
        kv("New wallet", &kp.pubkey().to_string());
        line("Fund it: fhe-cli airdrop  or https://faucet.solana.com");
    }

    doctor(cfg)?;

    if !Path::new(REGISTRY_FILE).exists() {
        line("Running setup (memo mode)...");
        setup(cfg)?;
    }

    submit_task(cfg, 0, value, None)?;
    ok("Demo complete");
    Ok(())
}

pub fn balance(cfg: &CliConfig) -> Result<(), Box<dyn Error>> {
    let kp = load_keypair(&cfg.wallet_path)?;
    let rpc = RpcClient::new(cfg.rpc_url.clone());
    let sol = get_balance_sol(&rpc, &kp.pubkey())?;
    kv("Wallet", &kp.pubkey().to_string());
    kv("Balance", &format!("{sol:.6} SOL"));
    Ok(())
}

pub fn airdrop(cfg: &CliConfig, sol: f64) -> Result<(), Box<dyn Error>> {
    title("Devnet Airdrop");
    let kp = load_keypair(&cfg.wallet_path)?;
    let rpc = RpcClient::new(cfg.rpc_url.clone());
    line(&format!("Requesting {sol} SOL for {}...", kp.pubkey()));
    match request_airdrop(&rpc, &kp.pubkey(), sol) {
        Ok(sig) => {
            tx_success(&sig.to_string());
            let new_bal = get_balance_sol(&rpc, &kp.pubkey())?;
            kv("New balance", &format!("{new_bal:.6} SOL"));
        }
        Err(e) => {
            fail(&format!("Airdrop failed: {e}"));
            line("Public devnet faucets are often rate-limited.");
            line("Try: https://faucet.solana.com (paste your wallet address)");
            return Err(e);
        }
    }
    Ok(())
}

pub fn history(cfg: &CliConfig, limit: usize) -> Result<(), Box<dyn Error>> {
    title("Transaction History");
    let kp = load_keypair(&cfg.wallet_path)?;
    let rpc = RpcClient::new(cfg.rpc_url.clone());
    let sigs = get_signatures(&rpc, &kp.pubkey(), limit)?;
    if sigs.is_empty() {
        warn("No transactions found for this wallet");
        return Ok(());
    }
    for (i, s) in sigs.iter().enumerate() {
        println!();
        kv(&format!("#{}", i + 1), &s.signature);
        if let Some(err) = &s.err {
            line(&format!("Error: {err:?}"));
        }
        output::solscan_devnet(&s.signature);
    }
    Ok(())
}

pub fn cache_list(cfg: &CliConfig) -> Result<(), Box<dyn Error>> {
    title("Ciphertext Cache");
    let cache = LocalCache::new(&cfg.cache_dir);
    let uris = cache.list()?;
    if uris.is_empty() {
        warn("Cache is empty — run: fhe-cli submit or fhe-cli encrypt");
        return Ok(());
    }
    for uri in uris {
        let bytes = cache.load(&uri)?;
        let hash = sha256_hex(&bytes);
        line(&format!("{uri}  ({} bytes, sha256={hash})", bytes.len()));
    }
    let size = cache.size()?;
    kv("Total cache size", &format!("{} bytes", size));
    Ok(())
}

pub fn cache_show(cfg: &CliConfig, hash_or_uri: &str) -> Result<(), Box<dyn Error>> {
    let uri = if hash_or_uri.starts_with("local://") {
        hash_or_uri.to_string()
    } else {
        format!("local://{}", hash_or_uri.trim_start_matches("0x"))
    };
    let cache = LocalCache::new(&cfg.cache_dir);
    let bytes = cache.load(&uri)?;
    kv("URI", &uri);
    kv("Size", &format!("{} bytes", bytes.len()));
    kv("SHA-256", &sha256_hex(&bytes));
    Ok(())
}

pub fn watch(cfg: &CliConfig, interval_secs: u64, limit: usize) -> Result<(), Box<dyn Error>> {
    title("Watch Wallet Activity");
    let kp = load_keypair(&cfg.wallet_path)?;
    kv("Wallet", &kp.pubkey().to_string());
    line(&format!("Polling every {interval_secs}s (Ctrl+C to stop)"));
    let rpc = RpcClient::new(cfg.rpc_url.clone());
    let mut seen = std::collections::HashSet::new();

    loop {
        if let Ok(sigs) = get_signatures(&rpc, &kp.pubkey(), limit) {
            for s in sigs.iter().rev() {
                if seen.insert(s.signature.clone()) {
                    println!();
                    ok(&format!("New activity: {}", s.signature));
                    output::solscan_devnet(&s.signature);
                }
            }
        }
        thread::sleep(Duration::from_secs(interval_secs));
    }
}

pub fn encrypt(cfg: &CliConfig, value: u32, out_path: &str) -> Result<(), Box<dyn Error>> {
    title("Encrypt FheUint32");
    let bytes = encrypt_u32(value, &cfg.key_dir)?;
    fs::write(out_path, &bytes)?;
    let cache = LocalCache::new(&cfg.cache_dir);
    let uri = cache.store(&bytes)?;
    kv("Value", &value.to_string());
    kv("Output file", out_path);
    kv("Bytes", &bytes.len().to_string());
    kv("SHA-256", &sha256_hex(&bytes));
    kv("Cache URI", &uri);
    ok("Encrypted and cached");
    Ok(())
}

pub fn submit_file(cfg: &CliConfig, file_path: &str, _op: u8) -> Result<(), Box<dyn Error>> {
    title("Submit Ciphertext File");
    let bytes = fs::read(file_path)?;
    kv("File", file_path);
    kv("Bytes", &bytes.len().to_string());
    kv("SHA-256", &sha256_hex(&bytes));
    let cache = LocalCache::new(&cfg.cache_dir);
    let uri = cache.store(&bytes)?;
    kv("Cache URI", &uri);

    if is_memo_mode(&cfg.program_id) {
        return submit_memo_with_uri(cfg, &uri);
    }

    warn("Coordinator mode: use fhe-cli submit-input with --value for inline ix");
    Err("submit-file is for memo mode. Set program to SPL Memo or use submit-input.".into())
}

pub fn keygen(cfg: &CliConfig, force: bool) -> Result<(), Box<dyn Error>> {
    title("FHE Key Generation");
    if fhestate_rs::keys::keys_exist(&cfg.key_dir) && !force {
        warn(&format!("Keys already exist in '{}'", cfg.key_dir));
        line("Use --force to regenerate");
        return Ok(());
    }
    let km = KeyManager::generate().map_err(|e| format!("Key generation failed: {e}"))?;
    km.save(&cfg.key_dir)
        .map_err(|e| format!("Failed to save keys: {e}"))?;
    ok(&format!("Keys saved to '{}'", cfg.key_dir));
    Ok(())
}

pub fn wallet_new(cfg: &CliConfig, out_path: Option<&str>) -> Result<(), Box<dyn Error>> {
    title("New Solana Wallet");
    let path = out_path.unwrap_or(&cfg.wallet_path);
    let kp = generate_wallet(path)?;
    ok("Wallet created");
    kv("Path", path);
    kv("Address", &kp.pubkey().to_string());
    line("Fund on devnet: fhe-cli airdrop 1");
    Ok(())
}

pub fn config_init(cfg: &CliConfig) -> Result<(), Box<dyn Error>> {
    save_config(cfg)?;
    ok(&format!("Wrote {CONFIG_FILE}"));
    Ok(())
}

pub fn flow_counter(cfg: &CliConfig, value: u32) -> Result<(), Box<dyn Error>> {
    title("Counter-Spy Flow");
    if is_memo_mode(&cfg.program_id) {
        warn("Program is SPL Memo — using demo submit (no coordinator PDA)");
        return demo(cfg, value);
    }
    ensure_fhe_keys(&cfg.key_dir)?;
    init_state(cfg)?;
    submit_input(cfg, 0, value, None)?;
    ok("Counter flow finished");
    Ok(())
}

pub fn overrides_from(
    rpc: Option<String>,
    program: Option<String>,
    wallet: Option<String>,
) -> ConfigOverrides {
    ConfigOverrides {
        rpc_url: rpc,
        program_id: program,
        wallet_path: wallet,
        ..Default::default()
    }
}
