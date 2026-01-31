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

    pub fn submit_task(ctx: Context<SubmitTask>, input_hash: [u8; 32], operation: u8) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        let task = &mut ctx.accounts.task;
        
        task.id = registry.task_count;
        task.submitter = ctx.accounts.submitter.key();
        task.input_hash = input_hash;
        task.operation = operation;
        task.status = TaskStatus::Pending;
        task.result_hash = [0u8; 32];
        task.executor = Pubkey::default();
        
        registry.task_count += 1;
        
        emit!(TaskSubmitted {
            task_id: task.id,
            submitter: task.submitter,
            input_hash,
            operation,
        });
        
        Ok(())
    }

    pub fn complete_task(ctx: Context<CompleteTask>, result_hash: [u8; 32]) -> Result<()> {
        let task = &mut ctx.accounts.task;
        let executor = &mut ctx.accounts.executor;
        
        require!(task.status == TaskStatus::Pending, CoordinatorError::TaskNotPending);
        require!(executor.active, CoordinatorError::ExecutorInactive);
        
        task.result_hash = result_hash;
        task.executor = ctx.accounts.executor_signer.key();
        task.status = TaskStatus::Completed;
        
        executor.tasks_completed += 1;
        
        emit!(TaskCompleted {
            task_id: task.id,
            executor: task.executor,
            result_hash,
        });
        
        Ok(())
    }

    pub fn challenge_task(ctx: Context<ChallengeTask>) -> Result<()> {
        let task = &mut ctx.accounts.task;
        
        require!(task.status == TaskStatus::Completed, CoordinatorError::TaskNotCompleted);
        
        task.status = TaskStatus::Challenged;
        
        emit!(TaskChallenged {
            task_id: task.id,
            challenger: ctx.accounts.challenger.key(),
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
pub struct CompleteTask<'info> {
    #[account(mut)]
    pub task: Account<'info, Task>,
    #[account(mut)]
    pub executor: Account<'info, Executor>,
    pub executor_signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ChallengeTask<'info> {
    #[account(mut)]
    pub task: Account<'info, Task>,
    pub challenger: Signer<'info>,
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
    pub input_hash: [u8; 32],
    pub operation: u8,
    pub status: TaskStatus,
    pub result_hash: [u8; 32],
    pub executor: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum TaskStatus {
    Pending,
    Completed,
    Challenged,
    Resolved,
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
    pub input_hash: [u8; 32],
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
