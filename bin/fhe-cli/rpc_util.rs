use solana_client::rpc_client::RpcClient;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Signature};
pub fn get_balance_sol(rpc: &RpcClient, pubkey: &Pubkey) -> Result<f64, Box<dyn std::error::Error>> {
    let lamports = rpc.get_balance(pubkey)?;
    Ok(lamports as f64 / LAMPORTS_PER_SOL as f64)
}

pub fn request_airdrop(
    rpc: &RpcClient,
    pubkey: &Pubkey,
    sol: f64,
) -> Result<Signature, Box<dyn std::error::Error>> {
    let lamports = (sol * LAMPORTS_PER_SOL as f64) as u64;
    let sig = rpc.request_airdrop(pubkey, lamports)?;
    let latest = rpc.get_latest_blockhash()?;
    rpc.confirm_transaction_with_spinner(&sig, &latest, rpc.commitment())?;
    Ok(sig)
}

pub fn get_signatures(
    rpc: &RpcClient,
    pubkey: &Pubkey,
    limit: usize,
) -> Result<Vec<solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature>, Box<dyn std::error::Error>>
{
    let mut sigs = rpc.get_signatures_for_address(pubkey)?;
    sigs.truncate(limit);
    Ok(sigs)
}

pub fn rpc_slot(rpc: &RpcClient) -> Result<u64, Box<dyn std::error::Error>> {
    Ok(rpc.get_slot()?)
}
