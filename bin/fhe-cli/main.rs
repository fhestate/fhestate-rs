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

    /// One-time setup: Initialize Registry and User State
    Setup {
        #[arg(long, default_value = "https://api.devnet.solana.com")]
        rpc_url: String,
        #[arg(long, default_value = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr")]
        program: String,
        #[arg(long, default_value = "deploy-wallet.json")]
        wallet: String,
    },

    /// Submit a task (Uses saved registry)
    Submit {
        #[arg(long, default_value = "https://api.devnet.solana.com")]
        rpc_url: String,
        #[arg(long, default_value = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr")]
        program: String,
        #[arg(long, default_value = "deploy-wallet.json")]
        wallet: String,
        #[arg(short, long)]
        value: u32,
        #[arg(long)]
        target: Option<String>,
    },

    /// Submit a small encrypted input directly (inline)
    SubmitInput {
        #[arg(long, default_value = "https://api.devnet.solana.com")]
        rpc_url: String,
        #[arg(long, default_value = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr")]
        program: String,
        #[arg(long, default_value = "deploy-wallet.json")]
        wallet: String,
        #[arg(short, long)]
        value: u32,
        #[arg(long)]
        target: Option<String>,
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

    /// Request a reveal (decryption) for an FHE task
    Reveal {
        #[arg(long, default_value = "https://api.devnet.solana.com")]
        rpc_url: String,
        #[arg(long, default_value = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr")]
        program: String,
        #[arg(long, default_value = "deploy-wallet.json")]
        wallet: String,
        #[arg(short, long)]
        task: String,
    },
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Setup {
            rpc_url,
            program,
            wallet,
        } => commands::setup(&rpc_url, &program, &wallet),
            op,
            value,
            target,
        } => commands::submit_task(&rpc_url, &program, &wallet, op, value, None, target.as_deref()),
            op,
            value,
            target,
        } => commands::submit_input(&rpc_url, &program, &wallet, op, value, target.as_deref()),
        Commands::InitState {
            rpc_url,
            program,
            wallet,
        } => commands::init_state(&rpc_url, &program, &wallet),
        Commands::Reveal {
            rpc_url,
            program,
            wallet,
            task,
        } => commands::reveal_task(&rpc_url, &program, &wallet, &task),
    };

    if let Err(e) = result {
        error!("Error: {}", e);
        process::exit(1);
    }
}
