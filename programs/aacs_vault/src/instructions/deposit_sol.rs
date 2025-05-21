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
    pub payer: Signer<'info>,

    #[account(
        mut,
        constraint = &vault.owner == payer.key @ErrorCode::NotVaultOwner 
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut, 
        seeds = [
            VAULT_SIGNER_PREFIX.as_ref(),
            vault.key().as_ref()
        ],
        bump,
    )]
    /// CHECK: this is signer of vault, holds SOL for that
    pub vault_signer: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn deposit_sol_handler(ctx: Context<DepositSol>, params: DepositSolParams) -> Result<()> {
    let payer_account = &ctx.accounts.payer;
    let vault_account = &ctx.accounts.vault;
    let vault_signer_account = &ctx.accounts.vault_signer;
    let system_program = &ctx.accounts.system_program;

    require_keys_eq!(
        vault_account.owner,
        payer_account.key(),
        ErrorCode::NotVaultOwner
    );

    require_gte!(
        payer_account.lamports(),
        params.amount,
        ErrorCode::NotEnoughLamports
    );

    let cpi_context = CpiContext::new(
        system_program.to_account_info(),
        Transfer {
            from: payer_account.to_account_info(),
            to: vault_signer_account.to_account_info(),
        },
    );

    transfer(cpi_context, params.amount)?;

    emit!(SolDeposited {
        amount: params.amount,
        depositor: payer_account.key(),
        vault: vault_account.key()
    });

    Ok(())
}
