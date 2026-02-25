use anchor_lang::prelude::*;

declare_id!("FHECord1111111111111111111111111111111111111");

#[program]
pub mod coordinator {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, min_stake: u64) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        registry.authority = ctx.accounts.authority.key();
        registry.min_stake = min_stake;
        registry.task_count = 0;
        registry.executor_count = 0;
        Ok(())
    }

    pub fn register_executor(ctx: Context<RegisterExecutor>, stake_amount: u64) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        let executor = &mut ctx.accounts.executor;
        
        require!(stake_amount >= registry.min_stake, CoordinatorError::InsufficientStake);
        
        // MOVEMENT: Actually transfer SOL to the executor account for staking
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.owner.to_account_info(),
                to: ctx.accounts.executor.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_context, stake_amount)?;

        executor.owner = ctx.accounts.owner.key();
        executor.stake = stake_amount;
        executor.active = true;
        executor.tasks_completed = 0;
        
        registry.executor_count += 1;
        
        emit!(ExecutorRegistered {
            executor: executor.owner,
            stake: stake_amount,
        });
        
        Ok(())
    }

    pub fn submit_task(ctx: Context<SubmitTask>, id: u64, input_hash: [u8; 32], input_uri: String, operation: u8, target_owner: Option<Pubkey>) -> Result<()> {
        let task = &mut ctx.accounts.task;
        let registry = &mut ctx.accounts.registry;

        require!(
            input_uri.starts_with("local://") || input_uri.starts_with("ipfs://"),
            CoordinatorError::InvalidStateUri
        );

        task.id = id;
        task.submitter = ctx.accounts.submitter.key();
        task.target_owner = target_owner.unwrap_or(ctx.accounts.submitter.key()); // Default to self if no target
        task.input_hash = input_hash;
        task.input_uri = input_uri;
        task.operation = operation;
        task.status = TaskStatus::Pending;
        task.result_hash = [0u8; 32];
        task.result_uri = String::default();
        task.executor = Pubkey::default();
        
        registry.task_count += 1;

        emit!(TaskSubmitted {
            task_id: id,
            submitter: task.submitter,
            target_owner: task.target_owner,
            operation,
        });

        Ok(())
    }

    /// Create a [`StateContainer`] PDA for the calling submitter.
    pub fn initialize_state(ctx: Context<InitializeState>) -> Result<()> {
        let container = &mut ctx.accounts.state_container;
        container.owner = ctx.accounts.submitter.key();
        container.state_hash = [0u8; 32];
        container.state_uri = String::new();
        container.version = 0;

        emit!(StateInitialized {
            owner: container.owner,
        });

        Ok(())
    }

    pub fn submit_input(
        ctx: Context<SubmitInput>,
        encrypted_data: Vec<u8>,
        operation: u8,
    ) -> Result<()> {
        let container = &mut ctx.accounts.state_container;
        let hash = anchor_lang::solana_program::hash::hash(&encrypted_data).to_bytes();
        
        container.state_hash = hash;
        container.state_uri = format!("inline://{}", hex::encode(hash));
        container.version += 1;

        emit!(TaskSubmitted {
            task_id: container.version,
            submitter: container.owner,
            target_owner: container.owner, // For inline input, target is always self
            operation,
        });

        Ok(())
    }

    pub fn update_state(
        ctx: Context<UpdateState>,
        previous_state_hash: [u8; 32],
        result_hash: [u8; 32],
        result_uri: String,
    ) -> Result<()> {
        let task = &mut ctx.accounts.task;
        let executor = &mut ctx.accounts.executor;
        let state_container = &mut ctx.accounts.state_container;

        require!(task.status == TaskStatus::Pending, CoordinatorError::TaskNotPending);
        require!(executor.active, CoordinatorError::ExecutorInactive);
        
        require!(
            state_container.state_hash == previous_state_hash,
            CoordinatorError::StateHashMismatch
        );

        task.result_hash = result_hash;
        task.result_uri = result_uri.clone();
        task.executor = executor.owner; // Attribute work to the executor account owner
        task.status = TaskStatus::Completed;
        
        state_container.state_hash = result_hash;
        state_container.state_uri = result_uri;
        state_container.version += 1;

        executor.tasks_completed += 1;

        emit!(TaskCompleted {
            task_id: task.id,
            executor: task.executor,
            result_hash,
        });

        emit!(StateUpdated {
            owner: state_container.owner,
            new_hash: result_hash,
            version: state_container.version,
        });

        Ok(())
    }

    /// Update a StateContainer directly (Fast-Path for Inline Ingestion).
    pub fn update_state_pda(
        ctx: Context<UpdateStatePda>,
        previous_state_hash: [u8; 32],
        result_hash: [u8; 32],
        result_uri: String,
    ) -> Result<()> {
        let state_container = &mut ctx.accounts.state_container;
        let executor = &mut ctx.accounts.executor;

        require!(executor.active, CoordinatorError::ExecutorInactive);
        require!(
            state_container.state_hash == previous_state_hash,
            CoordinatorError::StateHashMismatch
        );

        state_container.state_hash = result_hash;
        state_container.state_uri = result_uri;
        state_container.version += 1;
        
        executor.tasks_completed += 1;

        emit!(StateUpdated {
            owner: state_container.owner,
            new_hash: result_hash,
            version: state_container.version,
        });

        Ok(())
    }

    pub fn request_reveal(ctx: Context<RequestReveal>) -> Result<()> {
        let task = &mut ctx.accounts.task;
        require!(task.status == TaskStatus::Completed, CoordinatorError::TaskNotCompleted);
        task.status = TaskStatus::RevealRequested;
        Ok(())
    }

    pub fn provide_reveal(ctx: Context<ProvideReveal>, reveal_data: String) -> Result<()> {
        let task = &mut ctx.accounts.task;
        require!(task.status == TaskStatus::RevealRequested, CoordinatorError::InvalidStatus);
        
        task.reveal_result = reveal_data;
        task.status = TaskStatus::Revealed;
        Ok(())
    }

    pub fn challenge_task(ctx: Context<ChallengeTask>) -> Result<()> {
        let task = &mut ctx.accounts.task;
        let executor = &mut ctx.accounts.executor;
        let challenger = &ctx.accounts.challenger;
        
        // SECURITY: Only the original submitter can challenge their own task results in V1
        require!(task.status == TaskStatus::Completed, CoordinatorError::InvalidStatus);
        require!(task.submitter == challenger.key(), CoordinatorError::ExecutorUnauthorized);
        
        // SLASHING: Transfer executor's stake to challenger
        let amount = executor.stake;
        **executor.to_account_info().lamports.borrow_mut() -= amount;
        **challenger.to_account_info().lamports.borrow_mut() += amount;
        
        executor.stake = 0;
        executor.active = false;
        task.status = TaskStatus::Challenged;
        
        emit!(TaskChallenged {
            task_id: task.id,
            challenger: challenger.key(),
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + Registry::INIT_SPACE)]
    pub registry: Account<'info, Registry>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterExecutor<'info> {
    #[account(mut)]
    pub registry: Account<'info, Registry>,
    #[account(init, payer = owner, space = 8 + Executor::INIT_SPACE)]
    pub executor: Account<'info, Executor>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitTask<'info> {
    #[account(mut)]
    pub registry: Account<'info, Registry>,
    #[account(init, payer = submitter, space = 8 + Task::INIT_SPACE)]
    pub task: Account<'info, Task>,
    #[account(mut)]
    pub submitter: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RequestReveal<'info> {
    #[account(mut, has_one = submitter)]
    pub task: Account<'info, Task>,
    pub submitter: Signer<'info>,
}

#[derive(Accounts)]
pub struct ProvideReveal<'info> {
    #[account(mut, has_one = executor)]
    pub task: Account<'info, Task>,
    pub executor: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateState<'info> {
    #[account(mut)]
    pub task: Account<'info, Task>,
    #[account(
        mut,
        has_one = owner @ CoordinatorError::ExecutorUnauthorized
    )]
    pub executor: Account<'info, Executor>,
    #[account(
        mut,
        seeds = [b"state", task.submitter.as_ref()],
        bump,
    )]
    pub state_container: Account<'info, StateContainer>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateStatePda<'info> {
    #[account(
        mut,
        seeds = [b"state", owner_key.as_ref()],
        bump,
    )]
    pub state_container: Account<'info, StateContainer>,
    /// The owner of the state container (not necessarily the signer).
    /// CHECK: Used only for PDA seeds.
    pub owner_key: UncheckedAccount<'info>,
    #[account(
        mut,
        has_one = owner @ CoordinatorError::ExecutorUnauthorized
    )]
    pub executor: Account<'info, Executor>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct ChallengeTask<'info> {
    #[account(mut)]
    pub task: Account<'info, Task>,
    pub challenger: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitializeState<'info> {
    #[account(
        init,
        payer = submitter,
        space = 8 + StateContainer::INIT_SPACE,
        seeds = [b"state", submitter.key().as_ref()],
        bump
    )]
    pub state_container: Account<'info, StateContainer>,
    #[account(mut)]
    pub submitter: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitInput<'info> {
    #[account(
        mut,
        seeds = [b"state", submitter.key().as_ref()],
        bump
    )]
    pub state_container: Account<'info, StateContainer>,
    #[account(mut)]
    pub submitter: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Registry {
    pub authority: Pubkey,
    pub min_stake: u64,
    pub task_count: u64,
    pub executor_count: u64,
}

#[account]
#[derive(InitSpace)]
pub struct Executor {
    pub owner: Pubkey,
    pub stake: u64,
    pub active: bool,
    pub tasks_completed: u64,
}

#[account]
#[derive(InitSpace)]
pub struct Task {
    pub id: u64,
    pub submitter: Pubkey,
    pub target_owner: Pubkey, // NEW: Support shared state updates
    pub input_hash: [u8; 32],
    #[max_len(128)]
    pub input_uri: String,
    pub operation: u8,
    pub status: TaskStatus,
    pub result_hash: [u8; 32],
    #[max_len(128)]
    pub result_uri: String,
    #[max_len(256)]
    pub reveal_result: String,
    pub executor: Pubkey,
}

/// Persistent encrypted state container — one PDA per submitter.
/// Seeds: [b"state", owner.key().as_ref()].
/// Holds the URI of the current encrypted FheUint32 ciphertext and
/// a SHA256 hash for proof-of-computation verification.
#[account]
#[derive(InitSpace)]
pub struct StateContainer {
    /// The submitter who owns this state account.
    pub owner: Pubkey,
    /// SHA256 of the current encrypted state ciphertext bytes.
    /// All-zeros = state has not been computed yet (uninitialised).
    pub state_hash: [u8; 32],
    /// URI pointing to the ciphertext in off-chain storage.
    /// Format: "local://<sha256_hex>" or "ipfs://<cid>".
    #[max_len(128)]
    pub state_uri: String,
    /// Monotonically incrementing version — incremented on each state update.
    pub version: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    RevealRequested,
    Revealed,
    Challenged,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

#[error_code]
pub enum CoordinatorError {
    #[msg("Insufficient stake amount")]
    InsufficientStake,
    #[msg("Task is not pending")]
    TaskNotPending,
    #[msg("Task is not completed")]
    TaskNotCompleted,
    #[msg("Executor is inactive")]
    ExecutorInactive,
    #[msg("StateContainer already initialised for this owner")]
    PdaAlreadyInitialized,
    #[msg("State URI must start with local:// or ipfs://")]
    InvalidStateUri,
    #[msg("Signer is not the owner of this executor account")]
    ExecutorUnauthorized,
    #[msg("Invalid task status for this operation")]
    InvalidStatus,
    #[msg("State hash mismatch! Deterministic chain broken.")]
    StateHashMismatch,
}

#[event]
pub struct ExecutorRegistered {
    pub executor: Pubkey,
    pub stake: u64,
}

#[event]
pub struct TaskSubmitted {
    pub task_id: u64,
    pub submitter: Pubkey,
    pub target_owner: Pubkey,
    pub operation: u8,
}

#[event]
pub struct TaskCompleted {
    pub task_id: u64,
    pub executor: Pubkey,
    pub result_hash: [u8; 32],
}

#[event]
pub struct TaskChallenged {
    pub task_id: u64,
    pub challenger: Pubkey,
}

/// Emitted when a new StateContainer PDA is successfully created for a submitter.
/// Clients can subscribe to this event to know when encrypted state is ready.
#[event]
pub struct StateInitialized {
    pub owner: Pubkey,
}

#[event]
pub struct StateUpdated {
    pub owner: Pubkey,
    pub new_hash: [u8; 32],
    pub version: u64,
}
