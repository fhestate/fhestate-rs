use sha2::{Digest, Sha256};
use solana_program_test::*;
use solana_sdk::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction, system_program,
    transaction::Transaction,
};
use std::str::FromStr;

// Program ID from lib.rs
const PROGRAM_ID: &str = "FHECord1111111111111111111111111111111111111";

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
        coordinator::entry(program_id, accounts, input)
    }
}

#[tokio::test]
async fn test_coordinator_full_flow() {
    let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();
    let program_test = ProgramTest::new(
        "coordinator",
        program_id,
        processor!(process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Create Registry Keypair
    let registry_keypair = Keypair::new();
    let registry_pubkey = registry_keypair.pubkey();

    // ----------------------------------------------------
    // 1. Initialize Registry
    // ----------------------------------------------------
    let min_stake = 100_000_000u64; // 0.1 SOL
    let mut data = get_discriminator("initialize").to_vec();
    data.extend_from_slice(&min_stake.to_le_bytes());

    let ix_init = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new(registry_pubkey, true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_init], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &registry_keypair], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    println!("Initialized Coordinator Registry successfully.");

    // ----------------------------------------------------
    // 2. Register Executor
    // ----------------------------------------------------
    let executor_owner = Keypair::new();
    let (executor_pda, _executor_bump) =
        Pubkey::find_program_address(&[b"executor", executor_owner.pubkey().as_ref()], &program_id);

    // Fund executor owner
    let transfer_ix = system_instruction::transfer(
        &payer.pubkey(),
        &executor_owner.pubkey(),
        500_000_000, // 0.5 SOL
    );
    let mut transaction = Transaction::new_with_payer(&[transfer_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let stake_amount = 200_000_000u64; // 0.2 SOL
    data = get_discriminator("register_executor").to_vec();
    data.extend_from_slice(&stake_amount.to_le_bytes());

    let ix_reg_exec = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new(registry_pubkey, false),
            AccountMeta::new(executor_pda, false),
            AccountMeta::new(executor_owner.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_reg_exec], Some(&executor_owner.pubkey()));
    let blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&executor_owner], blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    println!("Registered Executor successfully.");

    // ----------------------------------------------------
    // 3. Initialize State Container for User
    // ----------------------------------------------------
    let user = Keypair::new();
    let (state_pda, _state_bump) =
        Pubkey::find_program_address(&[b"state", user.pubkey().as_ref()], &program_id);

    // Fund user
    let fund_user_ix = system_instruction::transfer(&payer.pubkey(), &user.pubkey(), 200_000_000);
    let mut transaction = Transaction::new_with_payer(&[fund_user_ix], Some(&payer.pubkey()));
    let blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&payer], blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    data = get_discriminator("initialize_state").to_vec();
    let ix_init_state = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_init_state], Some(&user.pubkey()));
    let blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&user], blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    println!("Initialized User State PDA successfully.");

    // ----------------------------------------------------
    // 4. Submit Task
    // ----------------------------------------------------
    let task_keypair = Keypair::new();
    let task_pubkey = task_keypair.pubkey();
    let task_id = 9999u64;
    let input_hash = [55u8; 32];
    let result_uri = "local://my_result_uri".to_string();
    let op = 1u8; // operation 1

    data = get_discriminator("submit_task").to_vec();
    data.extend_from_slice(&task_id.to_le_bytes());
    data.extend_from_slice(&input_hash);
    // Serialize String using Anchor serialization rules (length prefix as u32 + bytes)
    let uri_bytes = result_uri.as_bytes();
    data.extend_from_slice(&(uri_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(uri_bytes);
    data.push(op);
    data.push(1); // Some(target)
    data.extend_from_slice(user.pubkey().as_ref());

    let ix_submit = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new(registry_pubkey, false),
            AccountMeta::new(task_pubkey, true),
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_submit], Some(&user.pubkey()));
    let blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&user, &task_keypair], blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    println!("Submitted Task successfully.");

    // ----------------------------------------------------
    // 5. Update State
    // ----------------------------------------------------
    let previous_state_hash = [0u8; 32];
    let result_hash = [88u8; 32];
    let new_result_uri = "ipfs://QmNewResultHash".to_string();

    data = get_discriminator("update_state").to_vec();
    data.extend_from_slice(&previous_state_hash);
    data.extend_from_slice(&result_hash);
    let new_uri_bytes = new_result_uri.as_bytes();
    data.extend_from_slice(&(new_uri_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(new_uri_bytes);

    let ix_update = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new(task_pubkey, false),
            AccountMeta::new(executor_pda, false),
            AccountMeta::new(state_pda, false),
            AccountMeta::new(executor_owner.pubkey(), true),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_update], Some(&executor_owner.pubkey()));
    let blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&executor_owner], blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Verify state container hash update
    let state_account = banks_client.get_account(state_pda).await.unwrap().unwrap();
    // In StateContainer, owner is at offset 8 (32 bytes), state_hash is at offset 40 (32 bytes)
    let state_hash = &state_account.data[40..72];
    assert_eq!(state_hash, &result_hash);

    println!("Updated Encrypted State Container successfully.");

    // ----------------------------------------------------
    // 6. Challenge Task (Slashing)
    // ----------------------------------------------------
    // Executor balance before slashing
    let executor_bal_before = banks_client.get_balance(executor_pda).await.unwrap();
    let user_bal_before = banks_client.get_balance(user.pubkey()).await.unwrap();

    data = get_discriminator("challenge_task").to_vec();

    let ix_challenge = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new(task_pubkey, false),
            AccountMeta::new(executor_pda, false),
            AccountMeta::new(user.pubkey(), true),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_challenge], Some(&user.pubkey()));
    let blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&user], blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Verify slashing occurred: executor stake transferred to challenger (user)
    let executor_bal_after = banks_client.get_balance(executor_pda).await.unwrap();
    let user_bal_after = banks_client.get_balance(user.pubkey()).await.unwrap();

    assert_eq!(executor_bal_before - executor_bal_after, stake_amount);
    assert!(user_bal_after > user_bal_before);

    println!("Challenged Task and Slashed Executor successfully!");
}
