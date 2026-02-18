mod commands;

use clap::{Parser, Subcommand};
use log::error;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "fhe-cli", version, about = "FHEstate Client Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate FHE keypair
    Keygen {
        /// Output directory
        #[arg(short, long, default_value = "fhe_keys")]
        out_dir: String,
    },
    /// Generate Solana wallet
    Wallet {
        /// Output file path
        #[arg(short, long, default_value = "deploy-wallet.json")]
        output: String,
    },
    /// Run local encrypted sentence proof
    Proof {
        /// RPC URL
        #[arg(long, default_value = "https://api.devnet.solana.com")]
        rpc_url: String,
    },
    /// Submit a task to the blockchain
    Submit {
        #[arg(long, default_value = "https://api.devnet.solana.com")]
        rpc_url: String,
        #[arg(long, default_value = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr")]
        program: String,
        #[arg(long, default_value = "deploy-wallet.json")]
        wallet: String,
        #[arg(short, long, default_value_t = 1)]
        op: u8,
        /// The real value to encrypt as FheUint32
        #[arg(short, long)]
        value: u32,
    },
    /// Initialize a StateContainer PDA for the user
    InitState {
        #[arg(long, default_value = "https://api.devnet.solana.com")]
        rpc_url: String,
        #[arg(long, default_value = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr")]
        program: String,
        #[arg(long, default_value = "deploy-wallet.json")]
        wallet: String,
    },
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Keygen { out_dir } => commands::keygen(&out_dir),
        Commands::Wallet { output } => commands::wallet(&output),
        Commands::Proof { rpc_url } => commands::proof(&rpc_url),
        Commands::Submit {
            rpc_url,
            program,
            wallet,
            op,
            value,
        } => commands::submit_task(&rpc_url, &program, &wallet, op, value),
        Commands::InitState {
            rpc_url,
            program,
            wallet,
        } => commands::init_state(&rpc_url, &program, &wallet),
    };

    if let Err(e) = result {
        error!("Error: {}", e);
        process::exit(1);
    }
}
