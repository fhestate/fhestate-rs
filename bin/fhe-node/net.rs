use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::error::Error;

#[allow(dead_code)]
pub struct ChainListener {
    client: RpcClient,
}

#[allow(dead_code)]
impl ChainListener {
    pub fn new(rpc_url: &str) -> Self {
        let client =
            RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
        Self { client }
    }

    pub fn get_client(&self) -> &RpcClient {
        &self.client
    }

    pub fn get_slot(&self) -> Result<u64, Box<dyn Error>> {
        Ok(self.client.get_slot()?)
    }

    pub fn get_balance(&self, pubkey: &Pubkey) -> Result<u64, Box<dyn Error>> {
        Ok(self.client.get_balance(pubkey)?)
    }

    pub fn get_program_accounts(
        &self,
        program_id: &Pubkey,
    ) -> Result<Vec<(Pubkey, Vec<u8>)>, Box<dyn Error>> {
        let accounts = self.client.get_program_accounts(program_id)?;
        Ok(accounts
            .into_iter()
            .map(|(pk, acc)| (pk, acc.data))
            .collect())
    }

    pub fn is_connected(&self) -> bool {
        self.client.get_health().is_ok()
    }
}
