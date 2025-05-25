use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};

use crate::{
    constants::VAULT_SIGNER_PREFIX, state::Vault, error::ErrorCode
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawSolParams {
    amount: u64
}

#[derive(Accounts)]
pub struct WithdrawSol<'info> {
    #[account(mut)]
    pub withdrawer: Signer<'info>,

    #[account(
        mut,
        constraint = vault.owner == withdrawer.key() @ErrorCode::NotVaultOwner
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(mut, 
        seeds = [
            VAULT_SIGNER_PREFIX.as_ref(),
            vault.key().as_ref()
        ],
        bump = vault.signer_bump
    )]
    /// CHECK: vault_signer is a PDA program signer. Data is never read or written to
    pub vault_signer: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn withdraw_sol_handler(ctx: Context<WithdrawSol>, params: WithdrawSolParams) -> Result<()> {
    let vault_account = &ctx.accounts.vault;
    let withdrawer_account = &ctx.accounts.withdrawer;
    let vault_signer_account = &ctx.accounts.vault_signer;
    let system_program = &ctx.accounts.system_program;

    let vault_key = vault_account.key();
    
    require_gte!(
        vault_signer_account.lamports(),
        params.amount,
        ErrorCode::NotEnoughLamports
    );

    let seeds = &[
        VAULT_SIGNER_PREFIX.as_ref(),
        vault_key.as_ref(),
        &[vault_account.signer_bump],
    ];
    
    let signer_seeds = &[&seeds[..]];

    let cpi_context = CpiContext::new_with_signer(
        system_program.to_account_info(),
        Transfer {
            from: vault_signer_account.to_account_info(),
            to: withdrawer_account.to_account_info(),
        },
        signer_seeds,
    );

    transfer(cpi_context, params.amount)?;

    Ok(())
}
