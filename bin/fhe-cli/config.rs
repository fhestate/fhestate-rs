use fhestate_rs::constants::{CACHE_DIR, DEFAULT_RPC, KEY_DIR};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const MEMO_PROGRAM_ID: &str = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";
pub const REGISTRY_FILE: &str = ".fhestate_registry";
pub const CONFIG_DIR: &str = ".fhestate";
pub const CONFIG_FILE: &str = ".fhestate/config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub rpc_url: String,
    pub program_id: String,
    pub wallet_path: String,
    pub key_dir: String,
    pub cache_dir: String,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            rpc_url: DEFAULT_RPC.to_string(),
            program_id: MEMO_PROGRAM_ID.to_string(),
            wallet_path: "deploy-wallet.json".to_string(),
            key_dir: KEY_DIR.to_string(),
            cache_dir: CACHE_DIR.to_string(),
        }
    }
}

#[derive(Default)]
pub struct ConfigOverrides {
    pub rpc_url: Option<String>,
    pub program_id: Option<String>,
    pub wallet_path: Option<String>,
    pub key_dir: Option<String>,
    pub cache_dir: Option<String>,
}

pub fn load_config(overrides: ConfigOverrides) -> CliConfig {
    let mut cfg = if PathBuf::from(CONFIG_FILE).exists() {
        let raw = fs::read_to_string(CONFIG_FILE).unwrap_or_default();
        serde_json::from_str(&raw).unwrap_or_default()
    } else {
        CliConfig::default()
    };

    if let Ok(v) = std::env::var("FHESTATE_RPC") {
        if !v.is_empty() {
            cfg.rpc_url = v;
        }
    }
    if let Ok(v) = std::env::var("FHESTATE_PROGRAM_ID") {
        if !v.is_empty() {
            cfg.program_id = v;
        }
    }
    if let Ok(v) = std::env::var("FHESTATE_WALLET_PATH") {
        if !v.is_empty() {
            cfg.wallet_path = v;
        }
    }
    if let Ok(v) = std::env::var("FHESTATE_KEY_DIR") {
        if !v.is_empty() {
            cfg.key_dir = v;
        }
    }
    if let Ok(v) = std::env::var("FHESTATE_CACHE_DIR") {
        if !v.is_empty() {
            cfg.cache_dir = v;
        }
    }

    if let Some(v) = overrides.rpc_url {
        cfg.rpc_url = v;
    }
    if let Some(v) = overrides.program_id {
        cfg.program_id = v;
    }
    if let Some(v) = overrides.wallet_path {
        cfg.wallet_path = v;
    }
    if let Some(v) = overrides.key_dir {
        cfg.key_dir = v;
    }
    if let Some(v) = overrides.cache_dir {
        cfg.cache_dir = v;
    }

    cfg
}

pub fn save_config(cfg: &CliConfig) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(CONFIG_DIR)?;
    fs::write(CONFIG_FILE, serde_json::to_string_pretty(cfg)?)?;
    Ok(())
}

pub fn is_memo_mode(program_id: &str) -> bool {
    program_id == MEMO_PROGRAM_ID
}

pub fn registry_mode() -> Result<String, Box<dyn std::error::Error>> {
    if !PathBuf::from(REGISTRY_FILE).exists() {
        return Ok("not_configured".to_string());
    }
    let s = fs::read_to_string(REGISTRY_FILE)?;
    Ok(s.trim().to_string())
}
