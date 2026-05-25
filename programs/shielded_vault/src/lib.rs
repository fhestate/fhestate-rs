use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("FHEVault1111111111111111111111111111111111111");

#[program]
pub mod shielded_vault {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        registry.total_liquidity = 0;
        registry.authority = ctx.accounts.authority.key();
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
        require!(ctx.accounts.authority.key() == ctx.accounts.registry.authority, VaultError::Unauthorized);
        
        // The FHE off-chain worker posts the blinded math results
        ctx.accounts.sender_account.balance_hash = new_sender_hash;
        ctx.accounts.receiver_account.balance_hash = new_receiver_hash;

        Ok(())
    }

    pub fn unshield_funds(ctx: Context<UnshieldFunds>, amount: u64, vault_bump: u8) -> Result<()> {
        // Off-chain FHE worker verifies the user actually has the FHE balance before authorizing this
        require!(ctx.accounts.authority.key() == ctx.accounts.registry.authority, VaultError::Unauthorized);

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
        space = 8 + 32 + 8,
        seeds = [b"vault_registry"],
        bump
    )]
    pub registry: Account<'info, VaultRegistry>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
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
    pub authority: Pubkey,
    pub total_liquidity: u64,
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

#[error_code]
pub enum VaultError {
    #[msg("Unauthorized off-chain worker")]
    Unauthorized,
}
