use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};

declare_id!("2WVfb7G4DuvwgkJ9wm5SSDypBLECTfmpL78CwAosE3AD");

#[program]
mod escrow {
    use super::*;

    pub fn create_job(
        ctx: Context<CreateJob>,
        job_id: String,
        title: String,
        pay: u64,
    ) -> Result<()> {
        let job = &mut ctx.accounts.job;
        job.id = job_id;
        job.title = title;
        job.pay = pay;
        job.employer = ctx.accounts.employer.key();
        job.worker = ctx.accounts.worker.key();  // worker is now a Signer
        job.status = JobStatus::Pending;

        // Transfer pay from the employer to the escrow account
        let ix = system_instruction::transfer(
            &ctx.accounts.employer.key(),
            &ctx.accounts.escrow_account.key(),
            pay,
        );
        invoke(
            &ix,
            &[
                ctx.accounts.employer.to_account_info(),
                ctx.accounts.escrow_account.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn approve_job_worker(ctx: Context<ApproveJobWorker>) -> Result<()> {
        let job = &mut ctx.accounts.job;
    
        // Only the worker approves the job
        job.worker_approved = true;
        job.status = JobStatus::Completed;
    
        // If both parties approved, transfer funds to the worker
        if job.worker_approved && job.employer_approved {
            // Make sure the worker's account and escrow account are included in the context
            complete_payment(
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.worker.to_account_info(),
                job.pay,
            )?;
        }
    
        Ok(())
    }

    pub fn approve_job_employer(ctx: Context<ApproveJobEmployer>) -> Result<()> {
        let job = &mut ctx.accounts.job;
    
        // Only the employer approves the job
        job.employer_approved = true;
        
        // If both parties approved, transfer funds to the worker
        if job.worker_approved && job.employer_approved {
            // Make sure the worker's account and escrow account are included in the context
            complete_payment(
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.worker.to_account_info(),
                job.pay,
            )?;
        }
    
        Ok(())
    }

    pub fn reject_job(ctx: Context<RejectJob>) -> Result<()> {
        let job = &mut ctx.accounts.job;
        job.status = JobStatus::Rejected;

        // If either party rejects, funds remain in the escrow
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateJob<'info> {
    #[account(init, payer = employer, space = 8 + 32 + 32 + 64 + 64 + 32 + 1)]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub employer: Signer<'info>,
    #[account(mut)]
    pub worker: Signer<'info>, // Changed to Signer<'info>
    #[account(mut)]
    pub escrow_account: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveJobWorker<'info> {
    #[account(mut)]
    pub job: Account<'info, Job>,
    #[account(mut, address = job.worker)] // Ensure the signer is the worker
    pub worker: Signer<'info>,            // Only the worker signs here
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub escrow_account: AccountInfo<'info>, // Only the escrow is accessed here
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveJobEmployer<'info> {
    #[account(mut)]
    pub job: Account<'info, Job>,
    #[account(mut, address = job.employer)] // Ensure the signer is the employer
    pub employer: Signer<'info>,            // Only the employer signs here
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub worker: AccountInfo<'info>, 
    #[account(mut)]
    // /// CHECK: This is not dangerous because we don't read or write from this account
    pub escrow_account: Signer<'info>, // Only the escrow is accessed here
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RejectJob<'info> {
    #[account(mut)]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub signer: Signer<'info>, // Either employer or worker
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Job {
    pub id: String,
    pub title: String,
    pub pay: u64,
    pub employer: Pubkey,
    pub worker: Pubkey,
    pub worker_approved: bool,
    pub employer_approved: bool,
    pub status: JobStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Completed,
    Rejected,
}

// Helper function to transfer funds after both approvals
fn complete_payment<'a>(
    escrow_account: AccountInfo<'a>,
    worker_account: AccountInfo<'a>,
    pay: u64,
) -> Result<()> {
    let ix = system_instruction::transfer(
        &escrow_account.key(),   // From escrow
        &worker_account.key(),   // To worker
        pay,
    );

    invoke(
        &ix,
        &[escrow_account.clone(), worker_account.clone()], // Pass cloned accounts
    )?;

    Ok(())
}