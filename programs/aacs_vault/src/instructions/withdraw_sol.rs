use anchor_lang::{prelude::*, solana_program::program::invoke_signed};

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
    pub recipient: Signer<'info>,

    #[account(
        mut,
        constraint = vault.owner == recipient.key() @ErrorCode::NotVaultOwner
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
    let recipient_account = &ctx.accounts.recipient;
    let vault_signer_account = &ctx.accounts.vault_signer;

    let vault_key = vault_account.key();
    
    require_gte!(
        vault_signer_account.lamports(),
        params.amount,
        ErrorCode::NotEnoughLamports
    );

    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &vault_signer_account.key(),
        &recipient_account.key(),
        params.amount,
    );

    let accounts = vec![
        recipient_account.to_account_info(),
        vault_signer_account.to_account_info(),
    ];

    let vault_signer_seeds = &[
        VAULT_SIGNER_PREFIX.as_ref(),
        vault_key.as_ref(),
        &[vault_account.signer_bump],
    ];

    invoke_signed(&ix, &accounts, &[vault_signer_seeds])?;

    Ok(())
}
