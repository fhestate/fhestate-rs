use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("D14VbLLPcqkkZ6p4M9UDs4xfNdtB1tQDUqi7ZTt89etC");

#[program]
pub mod shielded_vault {
    use super::*;

    pub fn close_registry(ctx: Context<CloseRegistry>) -> Result<()> {
        let registry = &ctx.accounts.registry;
        let data = registry.try_borrow_data()?;
        require!(data.len() >= 40, VaultError::Unauthorized);
        let stored_admin = Pubkey::new_from_array(data[8..40].try_into().unwrap());
        require!(ctx.accounts.admin.key() == stored_admin, VaultError::Unauthorized);
        drop(data);
        
        let dest_starting_lamports = ctx.accounts.admin.lamports();
        **ctx.accounts.admin.lamports.borrow_mut() = dest_starting_lamports.checked_add(registry.lamports()).unwrap();
        **registry.lamports.borrow_mut() = 0;
        
        let mut data = registry.try_borrow_mut_data()?;
        for byte in data.iter_mut() {
            *byte = 0;
        }
        Ok(())
    }

    pub fn initialize_vault(ctx: Context<InitializeVault>, attestation_authority: Pubkey) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        registry.admin = ctx.accounts.authority.key();
        registry.attestation_authority = attestation_authority;
        registry.total_liquidity = 0;
        registry.approved_mrenclave = [0; 32];
        Ok(())
    }

    pub fn update_attestation_authority(ctx: Context<UpdateAttestationAuthority>, new_authority: Pubkey) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        require!(ctx.accounts.admin.key() == registry.admin, VaultError::Unauthorized);
        registry.attestation_authority = new_authority;
        Ok(())
    }

    pub fn update_approved_mrenclave(ctx: Context<UpdateApprovedMrenclave>, new_mrenclave: [u8; 32]) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        require!(ctx.accounts.admin.key() == registry.admin, VaultError::Unauthorized);
        registry.approved_mrenclave = new_mrenclave;
        Ok(())
    }

    pub fn initialize_account(ctx: Context<InitializeAccount>) -> Result<()> {
        let account = &mut ctx.accounts.encrypted_account;
        account.owner = ctx.accounts.owner.key();
        account.balance_hash = [0; 32];
        Ok(())
    }

    pub fn shield_funds(ctx: Context<ShieldFunds>, amount: u64) -> Result<()> {
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        );
        transfer(cpi_context, amount)?;
        
        let registry = &mut ctx.accounts.registry;
        registry.total_liquidity = registry.total_liquidity.checked_add(amount).unwrap();
        
        // The off-chain FHE worker listens for this event to physically add the amount 
        // to their encrypted FHE balance off-chain and submit it.
        emit!(ShieldEvent {
            user: ctx.accounts.user.key(),
            amount,
        });

        Ok(())
    }

    pub fn execute_transfer_fhe(
        ctx: Context<ExecuteTransferFhe>,
        new_sender_hash: [u8; 32],
        new_receiver_hash: [u8; 32],
    ) -> Result<()> {
        require!(ctx.accounts.authority.key() == ctx.accounts.registry.admin, VaultError::Unauthorized);
        
        // The FHE off-chain worker posts the blinded math results
        ctx.accounts.sender_account.balance_hash = new_sender_hash;
        ctx.accounts.receiver_account.balance_hash = new_receiver_hash;

        Ok(())
    }

    pub fn unshield_funds(ctx: Context<UnshieldFunds>, amount: u64, vault_bump: u8) -> Result<()> {
        // Off-chain FHE worker verifies the user actually has the FHE balance before authorizing this
        require!(ctx.accounts.authority.key() == ctx.accounts.registry.admin, VaultError::Unauthorized);

        let registry = &mut ctx.accounts.registry;
        registry.total_liquidity = registry.total_liquidity.checked_sub(amount).unwrap();

        let seeds = &["vault_auth".as_bytes(), &[vault_bump]];
        let signer = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            },
            signer,
        );
        transfer(cpi_context, amount)?;

        Ok(())
    }

    pub fn register_enclave(ctx: Context<RegisterEnclave>, enclave_key: Pubkey) -> Result<()> {
        let registry = &ctx.accounts.registry;
        require!(ctx.accounts.authority.key() == registry.admin, VaultError::Unauthorized);

        // Introspect instructions sysvar to verify the Ed25519 precompile instruction signature
        let current_index = anchor_lang::solana_program::sysvar::instructions::load_current_index_checked(
            &ctx.accounts.instructions
        )? as usize;
        
        require!(current_index >= 1, VaultError::InvalidEd25519Instruction);
        
        let precompile_ix = anchor_lang::solana_program::sysvar::instructions::load_instruction_at_checked(
            (current_index - 1) as usize,
            &ctx.accounts.instructions
        )?;
        
        require!(
            precompile_ix.program_id == anchor_lang::solana_program::ed25519_program::ID,
            VaultError::InvalidEd25519Instruction
        );
        
        let data = &precompile_ix.data;
        require!(data.len() >= 144, VaultError::InvalidEd25519Instruction);
        
        let pubkey_offset = u16::from_le_bytes([data[6], data[7]]) as usize;
        let message_offset = u16::from_le_bytes([data[10], data[11]]) as usize;
        let message_size = u16::from_le_bytes([data[12], data[13]]) as usize;
        
        // Task 3: Message is now 64 bytes: [enclave_key (32 bytes) | mrenclave (32 bytes)]
        require!(message_size == 64, VaultError::InvalidAttestationMessage);
        require!(data.len() >= pubkey_offset + 32, VaultError::InvalidEd25519Instruction);
        require!(data.len() >= message_offset + 64, VaultError::InvalidEd25519Instruction);
        
        let signer_pubkey = Pubkey::new_from_array(data[pubkey_offset..pubkey_offset + 32].try_into().unwrap());
        let signed_enclave_key = Pubkey::new_from_array(data[message_offset..message_offset + 32].try_into().unwrap());
        let signed_mrenclave: [u8; 32] = data[message_offset + 32..message_offset + 64].try_into().unwrap();
        
        // Verify that the signer matches the registry's attestation authority
        require!(signer_pubkey == registry.attestation_authority, VaultError::Unauthorized);
        
        // Verify that the signed enclave key matches target enclave_key
        require!(signed_enclave_key == enclave_key, VaultError::EnclaveKeyMismatch);

        // Verify that the signed mrenclave matches approved mrenclave
        require!(signed_mrenclave == registry.approved_mrenclave, VaultError::InvalidMrenclave);

        let enclave_account = &mut ctx.accounts.enclave_account;
        enclave_account.enclave_key = enclave_key;
        enclave_account.is_active = true;
        Ok(())
    }

    pub fn toggle_enclave(ctx: Context<ToggleEnclave>, is_active: bool) -> Result<()> {
        let registry = &ctx.accounts.registry;
        require!(ctx.accounts.authority.key() == registry.admin, VaultError::Unauthorized);

        let enclave_account = &mut ctx.accounts.enclave_account;
        enclave_account.is_active = is_active;
        Ok(())
    }

    pub fn execute_transfer_fhe_tee(
        ctx: Context<ExecuteTransferFheTee>,
        new_sender_hash: [u8; 32],
        new_receiver_hash: [u8; 32],
    ) -> Result<()> {
        let enclave_account = &ctx.accounts.enclave_account;
        require!(enclave_account.is_active, VaultError::UnauthorizedEnclave);
        require!(ctx.accounts.enclave_signer.key() == enclave_account.enclave_key, VaultError::UnauthorizedEnclave);

        ctx.accounts.sender_account.balance_hash = new_sender_hash;
        ctx.accounts.receiver_account.balance_hash = new_receiver_hash;
        Ok(())
    }

    pub fn unshield_funds_tee(ctx: Context<UnshieldFundsTee>, amount: u64, vault_bump: u8) -> Result<()> {
        let enclave_account = &ctx.accounts.enclave_account;
        require!(enclave_account.is_active, VaultError::UnauthorizedEnclave);
        require!(ctx.accounts.enclave_signer.key() == enclave_account.enclave_key, VaultError::UnauthorizedEnclave);

        let registry = &mut ctx.accounts.registry;
        registry.total_liquidity = registry.total_liquidity.checked_sub(amount).unwrap();

        let seeds = &["vault_auth".as_bytes(), &[vault_bump]];
        let signer = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            },
            signer,
        );
        transfer(cpi_context, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 8 + 32,
        seeds = [b"vault_registry"],
        bump
    )]
    pub registry: Account<'info, VaultRegistry>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAttestationAuthority<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault_registry"],
        bump
    )]
    pub registry: Account<'info, VaultRegistry>,
}

#[derive(Accounts)]
pub struct UpdateApprovedMrenclave<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault_registry"],
        bump
    )]
    pub registry: Account<'info, VaultRegistry>,
}

#[derive(Accounts)]
pub struct InitializeAccount<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32,
        seeds = [b"enc_account", owner.key().as_ref()],
        bump
    )]
    pub encrypted_account: Account<'info, EncryptedAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ShieldFunds<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    /// CHECK: PDA vault holding SOL
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub registry: Account<'info, VaultRegistry>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteTransferFhe<'info> {
    pub authority: Signer<'info>,
    pub registry: Account<'info, VaultRegistry>,
    #[account(mut)]
    pub sender_account: Account<'info, EncryptedAccount>,
    #[account(mut)]
    pub receiver_account: Account<'info, EncryptedAccount>,
}

#[derive(Accounts)]
pub struct UnshieldFunds<'info> {
    pub authority: Signer<'info>, // FHE off-chain worker authorizes this after verifying balance
    #[account(mut)]
    pub registry: Account<'info, VaultRegistry>,
    #[account(mut)]
    /// CHECK: PDA vault holding SOL
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub user: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct VaultRegistry {
    pub admin: Pubkey,
    pub attestation_authority: Pubkey,
    pub total_liquidity: u64,
    pub approved_mrenclave: [u8; 32],
}

#[account]
pub struct EncryptedAccount {
    pub owner: Pubkey,
    pub balance_hash: [u8; 32],
}

#[event]
pub struct ShieldEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[derive(Accounts)]
#[instruction(enclave_key: Pubkey)]
pub struct RegisterEnclave<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub registry: Account<'info, VaultRegistry>,
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 1,
        seeds = [b"enclave", enclave_key.as_ref()],
        bump
    )]
    pub enclave_account: Account<'info, EnclaveAccount>,
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: Instructions sysvar
    pub instructions: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ToggleEnclave<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub registry: Account<'info, VaultRegistry>,
    #[account(mut)]
    pub enclave_account: Account<'info, EnclaveAccount>,
}

#[derive(Accounts)]
pub struct ExecuteTransferFheTee<'info> {
    pub enclave_signer: Signer<'info>,
    #[account(
        seeds = [b"enclave", enclave_signer.key().as_ref()],
        bump
    )]
    pub enclave_account: Account<'info, EnclaveAccount>,
    #[account(mut)]
    pub sender_account: Account<'info, EncryptedAccount>,
    #[account(mut)]
    pub receiver_account: Account<'info, EncryptedAccount>,
}

#[derive(Accounts)]
pub struct UnshieldFundsTee<'info> {
    pub enclave_signer: Signer<'info>,
    #[account(
        seeds = [b"enclave", enclave_signer.key().as_ref()],
        bump
    )]
    pub enclave_account: Account<'info, EnclaveAccount>,
    #[account(mut)]
    pub registry: Account<'info, VaultRegistry>,
    #[account(mut)]
    /// CHECK: PDA vault holding SOL
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub user: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct EnclaveAccount {
    pub enclave_key: Pubkey,
    pub is_active: bool,
}

#[derive(Accounts)]
pub struct CloseRegistry<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault_registry"],
        bump
    )]
    /// CHECK: Checked in instruction body
    pub registry: UncheckedAccount<'info>,
}

#[error_code]
pub enum VaultError {
    #[msg("Unauthorized off-chain worker")]
    Unauthorized,
    #[msg("Unauthorized or inactive TEE enclave")]
    UnauthorizedEnclave,
    #[msg("Invalid Ed25519 signature precompile instruction")]
    InvalidEd25519Instruction,
    #[msg("Invalid attestation message length (expected 64 bytes)")]
    InvalidAttestationMessage,
    #[msg("Enclave key in attestation signature does not match target enclave key")]
    EnclaveKeyMismatch,
    #[msg("Enclave's code measurement (MRENCLAVE) does not match the approved version")]
    InvalidMrenclave,
}
