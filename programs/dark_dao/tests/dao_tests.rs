use sha2::{Digest, Sha256};
use solana_program_test::*;
use solana_sdk::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction, system_program,
    transaction::Transaction,
};
use std::str::FromStr;

// Program ID from lib.rs
const PROGRAM_ID: &str = "DarkDAo1111111111111111111111111111111111111";

fn get_discriminator(name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("global:{}", name).as_bytes());
    let result = hasher.finalize();
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&result[..8]);
    discriminator
}

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    unsafe {
        let accounts = std::mem::transmute(accounts);
        dark_dao::entry(program_id, accounts, input)
    }
}

#[tokio::test]
async fn test_dark_dao_full_flow() {
    let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();
    let program_test = ProgramTest::new(
        "dark_dao",
        program_id,
        processor!(process_instruction),
    );

    let mut context = program_test.start_with_context().await;
    let payer = &context.payer;
    let recent_blockhash = context.last_blockhash;

    // Derive Dao Config PDA
    let (config_pda, _config_bump) = Pubkey::find_program_address(&[b"config"], &program_id);

    // ----------------------------------------------------
    // 1. Initialize Dao
    // ----------------------------------------------------
    let data = get_discriminator("initialize").to_vec();
    let ix_init = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new(config_pda, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_init], Some(&payer.pubkey()));
    transaction.sign(&[payer], recent_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    println!("Initialized DAO Config successfully.");

    // ----------------------------------------------------
    // 2. Authorize Worker
    // ----------------------------------------------------
    let worker = Keypair::new();
    let (worker_record_pda, _worker_record_bump) =
        Pubkey::find_program_address(&[b"worker", worker.pubkey().as_ref()], &program_id);

    let mut data = get_discriminator("authorize_worker").to_vec();
    data.extend_from_slice(worker.pubkey().as_ref());
    let ix_auth_worker = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new_readonly(config_pda, false),
            AccountMeta::new(worker_record_pda, false),
            AccountMeta::new_readonly(worker.pubkey(), false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_auth_worker], Some(&payer.pubkey()));
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[payer], blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    println!("Authorized Worker successfully.");

    // ----------------------------------------------------
    // 3. Create Proposal
    // ----------------------------------------------------
    let proposal_keypair = Keypair::new();
    let proposal_pubkey = proposal_keypair.pubkey();
    let (tally_pda, _tally_bump) =
        Pubkey::find_program_address(&[b"tally", proposal_pubkey.as_ref()], &program_id);

    let voting_period = 60i64; // 1 minute
    let description = "Build FHE-based confidential state updates on Solana".to_string();

    let mut data = get_discriminator("create_proposal").to_vec();
    // Serialize String using Anchor serialization rules (length prefix as u32 + bytes)
    let desc_bytes = description.as_bytes();
    data.extend_from_slice(&(desc_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(desc_bytes);
    data.extend_from_slice(&voting_period.to_le_bytes());

    let ix_create_prop = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new(proposal_pubkey, true),
            AccountMeta::new(tally_pda, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_create_prop], Some(&payer.pubkey()));
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[payer, &proposal_keypair], blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    println!("Created Proposal successfully.");

    // ----------------------------------------------------
    // 4. Cast Encrypted Vote
    // ----------------------------------------------------
    let voter = Keypair::new();
    let (vote_record_pda, _vote_bump) = Pubkey::find_program_address(
        &[b"vote", proposal_pubkey.as_ref(), voter.pubkey().as_ref()],
        &program_id,
    );

    // Fund voter
    let fund_voter = system_instruction::transfer(&payer.pubkey(), &voter.pubkey(), 200_000_000);
    let mut transaction = Transaction::new_with_payer(&[fund_voter], Some(&payer.pubkey()));
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[payer], blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    let encrypted_vote = vec![1u8, 2u8, 3u8, 4u8]; // mock encrypted vote bytes
    let mut data = get_discriminator("cast_encrypted_vote").to_vec();
    // Vec<u8> is serialized as u32 length prefix + bytes
    data.extend_from_slice(&(encrypted_vote.len() as u32).to_le_bytes());
    data.extend_from_slice(&encrypted_vote);

    let ix_cast_vote = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new(proposal_pubkey, false),
            AccountMeta::new(vote_record_pda, false),
            AccountMeta::new(voter.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_cast_vote], Some(&voter.pubkey()));
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&voter], blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    println!("Cast Encrypted Vote successfully.");

    // ----------------------------------------------------
    // 5. Update Tally (Worker)
    // ----------------------------------------------------
    // Fund worker
    let fund_worker = system_instruction::transfer(&payer.pubkey(), &worker.pubkey(), 100_000_000);
    let mut transaction = Transaction::new_with_payer(&[fund_worker], Some(&payer.pubkey()));
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[payer], blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    let new_state_hash = [99u8; 32];
    let new_state_uri = "ipfs://QmMyTallyHash".to_string();

    let mut data = get_discriminator("update_tally").to_vec();
    data.extend_from_slice(&new_state_hash);
    let uri_bytes = new_state_uri.as_bytes();
    data.extend_from_slice(&(uri_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(uri_bytes);

    let ix_update_tally = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new_readonly(proposal_pubkey, false),
            AccountMeta::new(tally_pda, false),
            AccountMeta::new_readonly(worker_record_pda, false),
            AccountMeta::new(worker.pubkey(), true),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_update_tally], Some(&worker.pubkey()));
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&worker], blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    println!("Updated Tally successfully.");

    // ----------------------------------------------------
    // 6. Warp clock to end of voting and Finalize Tally
    // ----------------------------------------------------
    let clock: Clock = context.banks_client.get_sysvar().await.unwrap();
    let mut new_clock = clock.clone();
    new_clock.unix_timestamp += voting_period + 10; // Warp beyond expiration
    context.set_sysvar(&new_clock);

    let final_result_hash = [101u8; 32];
    let final_result_uri = "ipfs://QmFinalTallyResult".to_string();

    let mut data = get_discriminator("finalize_tally").to_vec();
    data.extend_from_slice(&final_result_hash);
    let final_uri_bytes = final_result_uri.as_bytes();
    data.extend_from_slice(&(final_uri_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(final_uri_bytes);

    let ix_finalize = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new(proposal_pubkey, false),
            AccountMeta::new(tally_pda, false),
            AccountMeta::new(payer.pubkey(), true),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_finalize], Some(&payer.pubkey()));
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[payer], blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    // Verify tally result hash updated on-chain
    let tally_account = context.banks_client.get_account(tally_pda).await.unwrap().unwrap();
    // In EncryptedTally, proposal is at 8 (32 bytes), state_hash is at 40 (32 bytes)
    let state_hash = &tally_account.data[40..72];
    assert_eq!(state_hash, &final_result_hash);

    println!("Finalized Tally and resolved DAO voting outcome successfully!");
}
