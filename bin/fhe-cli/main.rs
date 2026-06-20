mod commands;
mod config;
mod crypto_util;
mod output;
mod rpc_util;
mod vault_ops;
mod wallet;

use clap::{Parser, Subcommand};
use commands::{overrides_from, *};
use config::{load_config, CliConfig, MEMO_PROGRAM_ID};
use fhestate_rs::constants::CRATE_VERSION;
use std::process;

#[derive(Parser, Debug)]
#[command(
    name = "fhe-cli",
    version = CRATE_VERSION,
    about = "FHESTATE developer CLI — encrypt, monitor, and submit Solana devnet transactions"
)]
struct Cli {
    /// Solana RPC URL
    #[arg(long, global = true, env = "FHESTATE_RPC")]
    rpc_url: Option<String>,

    /// Program ID (default: SPL Memo for devnet demo)
    #[arg(long, global = true, env = "FHESTATE_PROGRAM_ID")]
    program: Option<String>,

    /// Path to Solana keypair JSON (byte array)
    #[arg(long, global = true, env = "FHESTATE_WALLET_PATH")]
    wallet: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// One-shot devnet demo: keys + encrypt + submit memo tx
    Demo {
        #[arg(short, long, default_value_t = 1337)]
        value: u32,
    },
    /// Health checks: keys, wallet, RPC, balance
    Doctor,
    /// Show keys, wallet, mode, cache summary
    Status,
    /// Write ~/.fhestate/config.json from current flags
    ConfigInit,
    /// First-time setup (keys + memo or coordinator)
    Setup,
    /// Encrypt a u32 and submit (memo or coordinator)
    Submit {
        #[arg(short, long, default_value_t = 0)]
        op: u8,
        #[arg(short, long, default_value_t = 1337)]
        value: u32,
        #[arg(long)]
        target: Option<String>,
    },
    /// Submit small inline ciphertext to coordinator
    SubmitInput {
        #[arg(short, long, default_value_t = 0)]
        op: u8,
        #[arg(short, long)]
        value: u32,
        #[arg(long)]
        target: Option<String>,
    },
    /// Submit an existing ciphertext .bin file (memo mode)
    SubmitFile {
        #[arg(long)]
        file: String,
        #[arg(short, long, default_value_t = 0)]
        op: u8,
    },
    /// Initialize StateContainer PDA
    InitState,
    /// Request reveal for a task
    Reveal {
        #[arg(short, long)]
        task: String,
    },
    /// Encrypt FheUint32 to file + cache
    Encrypt {
        #[arg(short, long, default_value_t = 1337)]
        value: u32,
        #[arg(short, long, default_value = "ciphertext.bin")]
        out: String,
    },
    /// Generate FHE keys
    Keygen {
        #[arg(long)]
        force: bool,
    },
    /// Create a new Solana wallet JSON file
    Wallet {
        #[command(subcommand)]
        cmd: WalletCommands,
    },
    /// Show wallet SOL balance
    Balance,
    /// Request devnet airdrop (may be rate-limited)
    Airdrop {
        #[arg(default_value_t = 1.0)]
        sol: f64,
    },
    /// Recent transaction signatures
    History {
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
    /// List cached ciphertext URIs
    Cache {
        #[command(subcommand)]
        cmd: CacheCommands,
    },
    /// Poll wallet for new transactions
    Watch {
        #[arg(short, long, default_value_t = 5)]
        interval: u64,
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
    },
    /// Counter-Spy: init state + submit (coordinator) or demo (memo)
    Flow {
        #[command(subcommand)]
        cmd: FlowCommands,
    },
    /// Homomorphic transfer balance hashes (JSON stdout)
    VaultTransferHashes {
        #[arg(long)]
        sender_balance_uri: Option<String>,
        #[arg(long)]
        receiver_balance_uri: Option<String>,
        #[arg(long)]
        amount_lamports: u64,
    },
    /// Post-shield deposit balance hash (JSON stdout)
    VaultDepositHash {
        #[arg(long)]
        balance_uri: Option<String>,
        #[arg(long)]
        deposit_lamports: u64,
    },
    /// Post-swap homomorphic balance hash (JSON stdout)
    VaultSwapHash {
        #[arg(long)]
        current_balance_uri: Option<String>,
        #[arg(long)]
        amount_in_lamports: u64,
        #[arg(long)]
        amount_out_lamports: u64,
    },
    /// Accumulate encrypted DAO vote into tally (JSON stdout)
    DaoTallyVote {
        #[arg(long)]
        tally_uri: Option<String>,
        #[arg(long)]
        vote_ciphertext_hex: String,
    },
    /// Store ciphertext bytes in LocalCache (JSON stdout)
    StoreCiphertext {
        #[arg(long)]
        ciphertext_hex: String,
    },
    /// Decrypt FheUint32 from cache URI or hex (JSON stdout)
    DecryptU32 {
        #[arg(long)]
        uri_or_hex: String,
    },
    /// Homomorphic spending guard check (JSON stdout)
    CheckSpending {
        #[arg(long)]
        daily_spend_uri: Option<String>,
        #[arg(long)]
        proposed_lamports: u64,
        #[arg(long, default_value_t = 1_000_000_000)]
        limit_lamports: u64,
    },
}

#[derive(Subcommand, Debug)]
enum WalletCommands {
    /// Create deploy-wallet.json (or --out path)
    New {
        #[arg(long)]
        out: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum CacheCommands {
    /// List all local:// entries
    List,
    /// Show one cache entry by hash or URI
    Show {
        hash: String,
    },
}

#[derive(Subcommand, Debug)]
enum FlowCommands {
    /// Initialize PDA + submit encrypted counter value
    Counter {
        #[arg(short, long, default_value_t = 1)]
        value: u32,
    },
}

fn cfg(cli: &Cli) -> CliConfig {
    load_config(overrides_from(
        cli.rpc_url.clone(),
        cli.program.clone(),
        cli.wallet.clone(),
    ))
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let config = cfg(&cli);

    let result = match cli.command {
        Commands::Demo { value } => demo(&config, value),
        Commands::Doctor => doctor(&config),
        Commands::Status => status(&config),
        Commands::ConfigInit => config_init(&config),
        Commands::Setup => setup(&config),
        Commands::Submit { op, value, target } => {
            submit_task(&config, op, value, target.as_deref())
        }
        Commands::SubmitInput { op, value, target } => {
            submit_input(&config, op, value, target.as_deref())
        }
        Commands::SubmitFile { file, op } => submit_file(&config, &file, op),
        Commands::InitState => init_state(&config),
        Commands::Reveal { task } => reveal_task(&config, &task),
        Commands::Encrypt { value, out } => encrypt(&config, value, &out),
        Commands::Keygen { force } => keygen(&config, force),
        Commands::Wallet { cmd } => match cmd {
            WalletCommands::New { out } => wallet_new(&config, out.as_deref()),
        },
        Commands::Balance => balance(&config),
        Commands::Airdrop { sol } => airdrop(&config, sol),
        Commands::History { limit } => history(&config, limit),
        Commands::Cache { cmd } => match cmd {
            CacheCommands::List => cache_list(&config),
            CacheCommands::Show { hash } => cache_show(&config, &hash),
        },
        Commands::Watch { interval, limit } => watch(&config, interval, limit),
        Commands::Flow { cmd } => match cmd {
            FlowCommands::Counter { value } => flow_counter(&config, value),
        },
        Commands::VaultTransferHashes {
            sender_balance_uri,
            receiver_balance_uri,
            amount_lamports,
        } => vault_ops::vault_transfer_hashes(
            &config,
            sender_balance_uri.as_deref(),
            receiver_balance_uri.as_deref(),
            amount_lamports,
        ),
        Commands::VaultDepositHash {
            balance_uri,
            deposit_lamports,
        } => vault_ops::vault_deposit_hash(&config, balance_uri.as_deref(), deposit_lamports),
        Commands::VaultSwapHash {
            current_balance_uri,
            amount_in_lamports,
            amount_out_lamports,
        } => vault_ops::vault_swap_hash(
            &config,
            current_balance_uri.as_deref(),
            amount_in_lamports,
            amount_out_lamports,
        ),
        Commands::DaoTallyVote {
            tally_uri,
            vote_ciphertext_hex,
        } => vault_ops::dao_tally_vote(&config, tally_uri.as_deref(), &vote_ciphertext_hex),
        Commands::StoreCiphertext { ciphertext_hex } => {
            vault_ops::store_ciphertext_hex(&config, &ciphertext_hex)
        }
        Commands::DecryptU32 { uri_or_hex } => vault_ops::decrypt_u32_from_uri(&config, &uri_or_hex),
        Commands::CheckSpending {
            daily_spend_uri,
            proposed_lamports,
            limit_lamports,
        } => vault_ops::check_spending(
            &config,
            daily_spend_uri.as_deref(),
            proposed_lamports,
            limit_lamports,
        ),
    };

    if let Err(e) = result {
        output::fail(&format!("{e}"));
        eprintln!();
        eprintln!("Tip: run `fhe-cli doctor` for help.");
        eprintln!("Default memo program: {MEMO_PROGRAM_ID}");
        process::exit(1);
    }
}
