mod commands;

use clap::{Parser, Subcommand};
use std::process;
use tracing::error;

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
        #[arg(short, long, default_value_t = 0)]
        op: u8,
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
        #[arg(short, long, default_value_t = 0)]
        op: u8,
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
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Setup {
            rpc_url,
            program,
            wallet,
        } => commands::setup(&rpc_url, &program, &wallet),
        Commands::Submit {
            rpc_url,
            program,
            wallet,
            op,
            value,
            target,
        } => commands::submit_task(&rpc_url, &program, &wallet, op, value, target.as_deref()),
        Commands::SubmitInput {
            rpc_url,
            program,
            wallet,
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
