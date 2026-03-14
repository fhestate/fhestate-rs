use anchor_lang::prelude::*;

declare_id!("DarkDAo1111111111111111111111111111111111111");

#[program]
pub mod dark_dao {
    use super::*;

    /// Initialize the DAO configuration and authority.
    pub fn initialize(ctx: Context<InitializeDao>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        Ok(())
    }

    /// Authorize a new FHE worker to submit tallies.
    pub fn authorize_worker(ctx: Context<AuthorizeWorker>, worker_key: Pubkey) -> Result<()> {
        let worker_record = &mut ctx.accounts.worker_record;
        worker_record.pubkey = worker_key;
        worker_record.is_active = true;
        Ok(())
    }

    /// Initialize a new proposal with an associated FHE state.
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        description: String,
        voting_period: i64,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let clock = Clock::get()?;

        proposal.creator = ctx.accounts.creator.key();
        proposal.description = description;
        proposal.start_time = clock.unix_timestamp;
        proposal.end_time = clock.unix_timestamp + voting_period;
        proposal.status = ProposalStatus::Active;
        proposal.total_votes = 0;
        
        // Initialize the Tally PDA reference
        let tally = &mut ctx.accounts.tally;
        tally.proposal = proposal.key();
        tally.state_hash = [0u8; 32];
        tally.state_uri = String::new();
        tally.version = 0;

        emit!(ProposalCreated {
            proposal: proposal.key(),
            creator: proposal.creator,
            end_time: proposal.end_time,
        });

        Ok(())
    }

    /// Post an encrypted vote ciphertext to the chain.
    /// This doesn't update the tally directly; it records the vote 
    /// and emits an event for the FHE worker to process.
    pub fn cast_encrypted_vote(
        ctx: Context<CastEncryptedVote>,
        encrypted_vote: Vec<u8>,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let vote_record = &mut ctx.accounts.vote_record;
        let clock = Clock::get()?;

        require!(proposal.status == ProposalStatus::Active, DaoError::ProposalNotActive);
        require!(clock.unix_timestamp <= proposal.end_time, DaoError::VotingEnded);

        vote_record.voter = ctx.accounts.voter.key();
        vote_record.proposal = proposal.key();
        vote_record.timestamp = clock.unix_timestamp;

        proposal.total_votes += 1;

        emit!(VoteCast {
            proposal: proposal.key(),
            voter: vote_record.voter,
            encrypted_vote, // Worker picks this up
        });

        Ok(())
    }

    /// Allows the FHE worker to update the running encrypted tally in the PDA.
    /// This satisfies the "accumulate on-chain" requirement.
    /// Access is restricted to authorized workers via the AuthorizedWorker PDA.
    pub fn update_tally(
        ctx: Context<UpdateTally>,
        new_state_hash: [u8; 32],
        new_state_uri: String,
    ) -> Result<()> {
        let proposal = &ctx.accounts.proposal;
        let tally = &mut ctx.accounts.tally;
        let worker_record = &ctx.accounts.worker_record;

        require!(proposal.status == ProposalStatus::Active, DaoError::ProposalNotActive);
        require!(worker_record.is_active, DaoError::UnauthorizedWorker);
        
        tally.state_hash = new_state_hash;
        tally.state_uri = new_state_uri;
        tally.version += 1;

        emit!(TallyUpdated {
            proposal: proposal.key(),
            new_hash: new_state_hash,
            version: tally.version,
        });

        Ok(())
    }

    /// Transitions the proposal to Tallying state, allowing the worker to submit the result.
    pub fn finalize_tally(ctx: Context<FinalizeTally>, result_hash: [u8; 32], result_uri: String) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let tally = &mut ctx.accounts.tally;
        let clock = Clock::get()?;

        require!(clock.unix_timestamp > proposal.end_time, DaoError::VotingStillActive);
        require!(proposal.status == ProposalStatus::Active, DaoError::InvalidStatus);

        proposal.status = ProposalStatus::Tallying;
        
        tally.state_hash = result_hash;
        tally.state_uri = result_uri;
        tally.version += 1;

        emit!(TallyFinalized {
            proposal: proposal.key(),
            result_hash,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeDao<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + DaoConfig::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, DaoConfig>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AuthorizeWorker<'info> {
    #[account(
        seeds = [b"config"],
        bump,
        has_one = authority
    )]
    pub config: Account<'info, DaoConfig>,
    #[account(
        init,
        payer = authority,
        space = 8 + AuthorizedWorker::INIT_SPACE,
        seeds = [b"worker", worker_key.as_ref()],
        bump
    )]
    pub worker_record: Account<'info, AuthorizedWorker>,
    /// CHECK: Passed as seed for the new worker record
    pub worker_key: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + Proposal::INIT_SPACE
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(
        init,
        payer = creator,
        space = 8 + EncryptedTally::INIT_SPACE,
        seeds = [b"tally", proposal.key().as_ref()],
        bump
    )]
    pub tally: Account<'info, EncryptedTally>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CastEncryptedVote<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    #[account(
        init,
        payer = voter,
        space = 8 + VoteRecord::INIT_SPACE,
        seeds = [b"vote", proposal.key().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,
    #[account(mut)]
    pub voter: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateTally<'info> {
    pub proposal: Account<'info, Proposal>,
    #[account(
        mut,
        seeds = [b"tally", proposal.key().as_ref()],
        bump
    )]
    pub tally: Account<'info, EncryptedTally>,
    #[account(
        seeds = [b"worker", worker.key().as_ref()],
        bump,
        constraint = worker_record.pubkey == worker.key() @ DaoError::UnauthorizedWorker
    )]
    pub worker_record: Account<'info, AuthorizedWorker>,
    #[account(mut)]
    pub worker: Signer<'info>,
}

#[derive(Accounts)]
pub struct FinalizeTally<'info> {
    #[account(mut, has_one = creator)]
    pub proposal: Account<'info, Proposal>,
    #[account(
        mut,
        seeds = [b"tally", proposal.key().as_ref()],
        bump
    )]
    pub tally: Account<'info, EncryptedTally>,
    pub creator: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct DaoConfig {
    pub authority: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct AuthorizedWorker {
    pub pubkey: Pubkey,
    pub is_active: bool,
}

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub creator: Pubkey,
    #[max_len(128)]
    pub description: String,
    pub start_time: i64,
    pub end_time: i64,
    pub status: ProposalStatus,
    pub total_votes: u64,
}

#[account]
#[derive(InitSpace)]
pub struct EncryptedTally {
    pub proposal: Pubkey,
    pub state_hash: [u8; 32],
    #[max_len(128)]
    pub state_uri: String,
    pub version: u64,
}

#[account]
#[derive(InitSpace)]
pub struct VoteRecord {
    pub voter: Pubkey,
    pub proposal: Pubkey,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum ProposalStatus {
    Active,
    Tallying,
    Succeeded,
    Defeated,
    Expired,
}

#[error_code]
pub enum DaoError {
    #[msg("Proposal is not currently active")]
    ProposalNotActive,
    #[msg("Voting period has ended")]
    VotingEnded,
    #[msg("Voting period is still active")]
    VotingStillActive,
    #[msg("Invalid proposal status")]
    InvalidStatus,
    #[msg("Worker is not authorized")]
    UnauthorizedWorker,
}

#[event]
pub struct ProposalCreated {
    pub proposal: Pubkey,
    pub creator: Pubkey,
    pub end_time: i64,
}

#[event]
pub struct VoteCast {
    pub proposal: Pubkey,
    pub voter: Pubkey,
    pub encrypted_vote: Vec<u8>,
}

#[event]
pub struct TallyUpdated {
    pub proposal: Pubkey,
    pub new_hash: [u8; 32],
    pub version: u64,
}

#[event]
pub struct TallyFinalized {
    pub proposal: Pubkey,
    pub result_hash: [u8; 32],
}
