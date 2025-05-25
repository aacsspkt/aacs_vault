pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

// pub use constants::*;
pub use instructions::*;

declare_id!("AacsqnF8FKT5ejrr6B9RsvPp8sErsTLJRM8mQQcgkKdL");

#[program]
pub mod aacs_vault {

    use super::*;

    pub fn create_vault(ctx: Context<CreateVault>, params: CreateVaultParams) -> Result<()> {
        create_vault_handler(ctx, params)
    }

    pub fn deposit_sol(ctx: Context<DepositSol>, params: DepositSolParams) -> Result<()> {
        deposit_sol_handler(ctx, params)
    }

    pub fn withdraw_sol(ctx: Context<WithdrawSol>, params: WithdrawSolParams) -> Result<()> {
        withdraw_sol_handler(ctx, params)
    }

    pub fn deposit_token(ctx: Context<DepositToken>, params: DepositTokenParams) -> Result<()> {
        deposit_token_handler(ctx, params)
    }

    pub fn withdraw_token(ctx: Context<WithdrawToken>, params: WithdrawTokenParams) -> Result<()> {
        withdraw_token_handler(ctx, params)
    }

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        params: CreateProposalParams,
    ) -> Result<()> {
        create_proposal_handler(ctx, params)
    }

    pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
        execute_proposal_handler(ctx)
    }

    pub fn execute_proposal_direct(
        ctx: Context<ExecuteProposalDirect>,
        params: ExecuteProposalDirectParams,
    ) -> Result<()> {
        execute_proposal_direct_handler(ctx, params)
    }
}
