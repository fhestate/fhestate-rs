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
const PROGRAM_ID: &str = "BN3ZTRhfkJEcWQBJviVdmYXySruGfxuE2jbV2FPiJ9qS";

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
        shielded_vault::entry(program_id, accounts, input)
    }
}

#[tokio::test]
async fn test_shielded_vault_full_flow() {
    let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();
    let program_test = ProgramTest::new(
        "shielded_vault",
        program_id,
        processor!(process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Derive registry PDA
    let (registry_pda, _registry_bump) =
        Pubkey::find_program_address(&[b"vault_registry"], &program_id);

    // Derive vault PDA (where the public SOL is deposited)
    let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault_auth"], &program_id);

    println!("Registry PDA: {}", registry_pda);
    println!("Vault PDA: {}", vault_pda);

    // ----------------------------------------------------
    // 1. Initialize Vault
    // ----------------------------------------------------
    let mut data = get_discriminator("initialize_vault").to_vec();

    let ix_init_vault = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new(registry_pda, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_init_vault], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    println!("Initialized Vault Registry successfully.");

    // ----------------------------------------------------
    // 2. Initialize Account for User A
    // ----------------------------------------------------
    let user_a = Keypair::new();
    let (enc_account_a, _) =
        Pubkey::find_program_address(&[b"enc_account", user_a.pubkey().as_ref()], &program_id);

    // Airdrop some SOL to User A so they can sign/pay
    let airdrop_ix = system_instruction::transfer(
        &payer.pubkey(),
        &user_a.pubkey(),
        1_000_000_000, // 1 SOL
    );
    let mut transaction = Transaction::new_with_payer(&[airdrop_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    data = get_discriminator("initialize_account").to_vec();
    let ix_init_acc_a = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new(enc_account_a, false),
            AccountMeta::new(user_a.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_init_acc_a], Some(&user_a.pubkey()));
    let blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&user_a], blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    println!("Initialized User A's Encrypted Account successfully.");

    // ----------------------------------------------------
    // 3. Shield Funds (Deposit SOL)
    // ----------------------------------------------------
    let shield_amount = 500_000_000u64; // 0.5 SOL
    data = get_discriminator("shield_funds").to_vec();
    data.extend_from_slice(&shield_amount.to_le_bytes());

    let ix_shield = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new(user_a.pubkey(), true),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(registry_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_shield], Some(&user_a.pubkey()));
    let blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&user_a], blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Verify vault balance and total liquidity
    let vault_balance = banks_client.get_balance(vault_pda).await.unwrap();
    assert_eq!(vault_balance, shield_amount);

    println!("Shielded 0.5 SOL successfully.");

    // ----------------------------------------------------
    // 4. Initialize Account for User B
    // ----------------------------------------------------
    let user_b = Keypair::new();
    let (enc_account_b, _) =
        Pubkey::find_program_address(&[b"enc_account", user_b.pubkey().as_ref()], &program_id);

    let airdrop_b = system_instruction::transfer(&payer.pubkey(), &user_b.pubkey(), 100_000_000);
    let mut transaction = Transaction::new_with_payer(&[airdrop_b], Some(&payer.pubkey()));
    let blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&payer], blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    data = get_discriminator("initialize_account").to_vec();
    let ix_init_acc_b = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new(enc_account_b, false),
            AccountMeta::new(user_b.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    let mut transaction = Transaction::new_with_payer(&[ix_init_acc_b], Some(&user_b.pubkey()));
    let blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&user_b], blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // ----------------------------------------------------
    // 5. Execute FHE confidential transfer (A -> B)
    // ----------------------------------------------------
    let new_a_hash: [u8; 32] = [11; 32];
    let new_b_hash: [u8; 32] = [22; 32];

    data = get_discriminator("execute_transfer_fhe").to_vec();
    data.extend_from_slice(&new_a_hash);
    data.extend_from_slice(&new_b_hash);

    // Reg authority is `payer.pubkey()`
    let ix_transfer = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new_readonly(payer.pubkey(), true),
            AccountMeta::new_readonly(registry_pda, false),
            AccountMeta::new(enc_account_a, false),
            AccountMeta::new(enc_account_b, false),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_transfer], Some(&payer.pubkey()));
    let blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&payer], blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Verify state updates
    let acc_a_data = banks_client
        .get_account(enc_account_a)
        .await
        .unwrap()
        .unwrap();
    // In EncryptedAccount, owner is at offset 8 (32 bytes), balance_hash is at offset 40 (32 bytes)
    let hash_a = &acc_a_data.data[40..72];
    assert_eq!(hash_a, &new_a_hash);

    let acc_b_data = banks_client
        .get_account(enc_account_b)
        .await
        .unwrap()
        .unwrap();
    let hash_b = &acc_b_data.data[40..72];
    assert_eq!(hash_b, &new_b_hash);

    println!("Executed FHE confidential transfer successfully.");

    // ----------------------------------------------------
    // 6. Unshield Funds (Withdraw)
    // ----------------------------------------------------
    let unshield_amount = 200_000_000u64; // 0.2 SOL
    data = get_discriminator("unshield_funds").to_vec();
    data.extend_from_slice(&unshield_amount.to_le_bytes());
    data.push(vault_bump);

    let ix_unshield = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new_readonly(payer.pubkey(), true),
            AccountMeta::new(registry_pda, false),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(user_a.pubkey(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut transaction = Transaction::new_with_payer(&[ix_unshield], Some(&payer.pubkey()));
    let blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&payer], blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Verify vault balance decreased
    let vault_balance_after = banks_client.get_balance(vault_pda).await.unwrap();
    assert_eq!(vault_balance_after, shield_amount - unshield_amount);

    println!("Unshielded 0.2 SOL successfully.");
}
