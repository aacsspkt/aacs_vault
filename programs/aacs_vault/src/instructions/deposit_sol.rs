use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    constants::VAULT_SIGNER_PREFIX, 
    error::ErrorCode, 
    events::SolDeposited, state::Vault};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositSolParams {
    pub amount: u64,
}

#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        mut,
        constraint = &vault.owner == depositor.key @ErrorCode::NotVaultOwner 
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut, 
        seeds = [
            VAULT_SIGNER_PREFIX.as_ref(),
            vault.key().as_ref()
        ],
        bump = vault.signer_bump,
    )]
    /// CHECK: this is signer of vault, holds SOL for that
    pub vault_signer: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn deposit_sol_handler(ctx: Context<DepositSol>, params: DepositSolParams) -> Result<()> {
    let depositor_account = &ctx.accounts.depositor;
    let vault_account = &ctx.accounts.vault;
    let vault_signer_account = &ctx.accounts.vault_signer;
    let system_program = &ctx.accounts.system_program;

    require_gte!(
        depositor_account.lamports(),
        params.amount,
        ErrorCode::NotEnoughLamports
    );

    let cpi_context = CpiContext::new(
        system_program.to_account_info(),
        Transfer {
            from: depositor_account.to_account_info(),
            to: vault_signer_account.to_account_info(),
        },
    );

    transfer(cpi_context, params.amount)?;

    emit!(SolDeposited {
        amount: params.amount,
        depositor: depositor_account.key(),
        vault: vault_account.key(),
        vault_signer: vault_signer_account.key()
    });

    Ok(())
}
