use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{ TransferChecked, transfer_checked, Token},
    token_interface:: {Mint, TokenAccount}
};

use crate::{
    constants::VAULT_SIGNER_PREFIX, 
    error::ErrorCode, 
    events::TokenWithdrawn, 
    state::Vault
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawTokenParams {
    amount: u64,
    decimals: u8
}

#[derive(Accounts)]
pub struct WithdrawToken<'info> {
    #[account(mut)]
    pub withdrawer: Signer<'info>,

     #[account(
        constraint = &vault.owner == withdrawer.key @ErrorCode::VaultNotOwned 
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account( 
        seeds = [
            VAULT_SIGNER_PREFIX.as_ref(),
            vault.key().as_ref()
        ],
        bump = vault.signer_bump,
    )]
    /// CHECK: this is signer of vault, holds SOL for that
    pub vault_signer: UncheckedAccount<'info>,

    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = vault_signer,
        // owner = token_program
    )]
    pub vault_signer_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    
     #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = withdrawer,
    )]
    pub withdrawer_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    
    pub token_program:  Program<'info, Token>,
    
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}

pub fn withdraw_token_handler(ctx: Context<WithdrawToken>, params: WithdrawTokenParams) -> Result<()> {
    let vault_account = &ctx.accounts.vault;
    let withdrawer_account = &ctx.accounts.withdrawer;
    let withdrawer_token_account = &ctx.accounts.withdrawer_token_account;
    let vault_signer_account = &ctx.accounts.vault_signer;
    let vault_signer_token_account = &ctx.accounts.vault_signer_token_account;
    let token_mint_account = &ctx.accounts.token_mint;
    let token_program = &ctx.accounts.token_program;

    require_gte!(vault_signer_token_account.amount, params.amount, ErrorCode::NotEnoughTokens);

    let vault_key = vault_account.key();
    
    let seeds = &[
        VAULT_SIGNER_PREFIX.as_ref(),
        vault_key.as_ref(),
        &[vault_account.signer_bump],
    ];
    
    let signer_seeds = &[&seeds[..]];

    let cpi_context = CpiContext::new_with_signer(
        token_program.to_account_info(),
        TransferChecked {
            from: vault_signer_token_account.to_account_info(),
            to: withdrawer_token_account.to_account_info(),
            authority: vault_signer_account.to_account_info(),
            mint: token_mint_account.to_account_info(),
        },
        signer_seeds
    );

    transfer_checked(cpi_context, params.amount, params.decimals)?;

    emit!(TokenWithdrawn {
        amount: params.amount,
        vault: vault_key,
        vault_signer: vault_signer_account.key(),
        vault_signer_token_account: vault_signer_token_account.key(),
        withdrawer: withdrawer_account.key(),
        withdrawer_token_account: withdrawer_token_account.key()
    });

    Ok(())
}
