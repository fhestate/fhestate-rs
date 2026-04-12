#[path = "net.rs"]
mod net;
#[path = "service.rs"]
mod service;

use clap::Parser;
use std::process;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(name = "fhe-node", version, about = "FHEstate Executor Node")]
struct Args {
    #[arg(short, long, default_value = "https://api.devnet.solana.com")]
    rpc_url: String,

    #[arg(short, long, default_value = "11111111111111111111111111111111")]
    program_id: String,

    #[arg(long, default_value = "deploy-wallet.json")]
    wallet: String,

    #[arg(long, default_value = "fhe_keys/server_key.bin")]
    server_key: String,

    #[arg(short, long, default_value_t = 1)]
    threads: u8,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let args = Args::parse();

    info!("FHEstate Executor Node v{}", env!("CARGO_PKG_VERSION"));
    info!("   RPC: {}", args.rpc_url);
    info!("   Program: {}", args.program_id);

    match service::ExecutorService::new(
        &args.rpc_url,
        &args.program_id,
        &args.wallet,
        &args.server_key,
    ) {
        Ok(executor) => {
            if let Err(e) = executor.run().await {
                error!("Executor error: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            error!("Startup failed: {}", e);
            process::exit(1);
        }
    }
}
