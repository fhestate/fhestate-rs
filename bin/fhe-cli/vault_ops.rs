//! Vault / DAO FHE operations — JSON output for backend integration.

use crate::config::CliConfig;
use crate::crypto_util::{ensure_fhe_keys, sha256_hex};
use fhestate_rs::constants::ops;
use fhestate_rs::keys::{activate_server_key, load_client_key, load_server_key};
use fhestate_rs::{LocalCache, StateTransition};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::error::Error;
use tfhe::prelude::*;
use tfhe::FheUint32;

#[derive(Serialize)]
struct TransferOut {
    sender_hash: String,
    receiver_hash: String,
    sender_uri: String,
    receiver_uri: String,
}

#[derive(Serialize)]
struct SwapHashOut {
    new_balance_hash: String,
    new_balance_uri: String,
}

#[derive(Serialize)]
struct TallyOut {
    new_state_hash: String,
    new_state_uri: String,
}

#[derive(Serialize)]
struct StoreOut {
    hash: String,
    uri: String,
}

#[derive(Serialize)]
struct DecryptOut {
    value: u32,
    uri: String,
}

fn load_balance_ct(
    cache: &LocalCache,
    uri: Option<&str>,
    key_dir: &str,
) -> Result<FheUint32, Box<dyn Error>> {
    ensure_fhe_keys(key_dir)?;
    if let Some(u) = uri {
        let trimmed = u.trim();
        if !trimmed.is_empty() {
            let bytes = cache.load(trimmed)?;
            return Ok(bincode::deserialize(&bytes)?);
        }
    }
    let client_key = load_client_key(key_dir)?;
    Ok(FheUint32::encrypt(0u32, &client_key))
}

fn store_ct(cache: &LocalCache, ct: &FheUint32) -> Result<(String, String), Box<dyn Error>> {
    let bytes = bincode::serialize(ct)?;
    let uri = cache.store(&bytes)?;
    let hash = sha256_hex(&bytes);
    Ok((hash, uri))
}

pub fn vault_transfer_hashes(
    cfg: &CliConfig,
    sender_uri: Option<&str>,
    receiver_uri: Option<&str>,
    amount_lamports: u64,
) -> Result<(), Box<dyn Error>> {
    ensure_fhe_keys(&cfg.key_dir)?;
    let server_key = load_server_key(&cfg.key_dir)?;
    activate_server_key(&server_key);

    let cache = LocalCache::new(&cfg.cache_dir);
    let client_key = load_client_key(&cfg.key_dir)?;

    let sender = load_balance_ct(&cache, sender_uri, &cfg.key_dir)?;
    let receiver = load_balance_ct(&cache, receiver_uri, &cfg.key_dir)?;
    let amount: u32 = amount_lamports.min(u32::MAX as u64) as u32;
    let amount_ct = FheUint32::encrypt(amount, &client_key);

    let sender_new = &sender - &amount_ct;
    let receiver_new = &receiver + &amount_ct;

    let (sender_hash, sender_out_uri) = store_ct(&cache, &sender_new)?;
    let (receiver_hash, receiver_out_uri) = store_ct(&cache, &receiver_new)?;

    let out = TransferOut {
        sender_hash,
        receiver_hash,
        sender_uri: sender_out_uri,
        receiver_uri: receiver_out_uri,
    };
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}

pub fn vault_deposit_hash(
    cfg: &CliConfig,
    balance_uri: Option<&str>,
    deposit_lamports: u64,
) -> Result<(), Box<dyn Error>> {
    ensure_fhe_keys(&cfg.key_dir)?;
    let server_key = load_server_key(&cfg.key_dir)?;
    activate_server_key(&server_key);

    let cache = LocalCache::new(&cfg.cache_dir);
    let client_key = load_client_key(&cfg.key_dir)?;

    let current = load_balance_ct(&cache, balance_uri, &cfg.key_dir)?;
    let deposit: u32 = deposit_lamports.min(u32::MAX as u64) as u32;
    let deposit_ct = FheUint32::encrypt(deposit, &client_key);
    let new_bal = &current + &deposit_ct;

    let (hash, uri) = store_ct(&cache, &new_bal)?;
    let out = SwapHashOut {
        new_balance_hash: hash,
        new_balance_uri: uri,
    };
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}

pub fn vault_swap_hash(
    cfg: &CliConfig,
    current_uri: Option<&str>,
    amount_in_lamports: u64,
    amount_out_lamports: u64,
) -> Result<(), Box<dyn Error>> {
    ensure_fhe_keys(&cfg.key_dir)?;
    let server_key = load_server_key(&cfg.key_dir)?;
    activate_server_key(&server_key);

    let cache = LocalCache::new(&cfg.cache_dir);
    let client_key = load_client_key(&cfg.key_dir)?;

    let current = load_balance_ct(&cache, current_uri, &cfg.key_dir)?;
    let amount_in: u32 = amount_in_lamports.min(u32::MAX as u64) as u32;
    let amount_out: u32 = amount_out_lamports.min(u32::MAX as u64) as u32;
    let in_ct = FheUint32::encrypt(amount_in, &client_key);
    let out_ct = FheUint32::encrypt(amount_out, &client_key);

    let after_out = &current - &in_ct + &out_ct;

    let (hash, uri) = store_ct(&cache, &after_out)?;
    let out = SwapHashOut {
        new_balance_hash: hash,
        new_balance_uri: uri,
    };
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}

pub fn dao_tally_vote(
    cfg: &CliConfig,
    tally_uri: Option<&str>,
    vote_ciphertext_hex: &str,
) -> Result<(), Box<dyn Error>> {
    ensure_fhe_keys(&cfg.key_dir)?;
    let server_key = load_server_key(&cfg.key_dir)?;
    activate_server_key(&server_key);

    let cache = LocalCache::new(&cfg.cache_dir);
    let vote_bytes = hex::decode(vote_ciphertext_hex.trim_start_matches("0x"))?;

    let state_uri = tally_uri.filter(|s| !s.is_empty());
    let (new_uri, hash_bytes) =
        StateTransition::apply(&cache, state_uri, &vote_bytes, ops::VOTE_TALLY)?;

    let out = TallyOut {
        new_state_hash: hex::encode(hash_bytes),
        new_state_uri: new_uri,
    };
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}

pub fn store_ciphertext_hex(cfg: &CliConfig, ciphertext_hex: &str) -> Result<(), Box<dyn Error>> {
    let bytes = hex::decode(ciphertext_hex.trim_start_matches("0x"))?;
    let cache = LocalCache::new(&cfg.cache_dir);
    let uri = cache.store(&bytes)?;
    let hash = sha256_hex(&bytes);
    let out = StoreOut { hash, uri };
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}

pub fn decrypt_u32_from_uri(cfg: &CliConfig, uri_or_hex: &str) -> Result<(), Box<dyn Error>> {
    ensure_fhe_keys(&cfg.key_dir)?;
    let client_key = load_client_key(&cfg.key_dir)?;
    let cache = LocalCache::new(&cfg.cache_dir);

    let bytes = if uri_or_hex.starts_with("local://") {
        cache.load(uri_or_hex)?
    } else {
        hex::decode(uri_or_hex.trim_start_matches("0x"))?
    };

    let ct: FheUint32 = bincode::deserialize(&bytes)?;
    let value: u32 = ct.decrypt(&client_key);

    let uri = if uri_or_hex.starts_with("local://") {
        uri_or_hex.to_string()
    } else {
        cache.store(&bytes)?
    };

    let out = DecryptOut { value, uri };
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}

pub fn check_spending(
    cfg: &CliConfig,
    daily_spend_uri: Option<&str>,
    proposed_lamports: u64,
    limit_lamports: u64,
) -> Result<(), Box<dyn Error>> {
    ensure_fhe_keys(&cfg.key_dir)?;
    let server_key = load_server_key(&cfg.key_dir)?;
    activate_server_key(&server_key);

    let cache = LocalCache::new(&cfg.cache_dir);
    let client_key = load_client_key(&cfg.key_dir)?;

    let current_spend = load_balance_ct(&cache, daily_spend_uri, &cfg.key_dir)?;
    let proposed: u32 = proposed_lamports.min(u32::MAX as u64) as u32;
    let limit: u32 = limit_lamports.min(u32::MAX as u64) as u32;

    let current_plain: u32 = current_spend.decrypt(&client_key);
    let allowed = current_plain.saturating_add(proposed) <= limit;

    #[derive(Serialize)]
    struct GuardOut {
        allowed: bool,
        reason: String,
    }

    let out = GuardOut {
        allowed,
        reason: if allowed {
            "within encrypted daily limit".to_string()
        } else {
            "proposed amount exceeds homomorphic daily spending limit".to_string()
        },
    };
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}
