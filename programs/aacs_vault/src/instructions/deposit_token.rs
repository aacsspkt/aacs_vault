use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{TransferChecked, transfer_checked, Token},
    token_interface::{Mint, TokenAccount}
};

use crate::{
    constants::VAULT_SIGNER_PREFIX, 
    error::ErrorCode, 
    events::TokenDeposited, 
    state::Vault
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositTokenParams {
    pub amount: u64,
    pub decimals: u8
}

#[derive(Accounts)]
pub struct DepositToken<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        constraint = &vault.owner == depositor.key @ErrorCode::VaultNotOwned 
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

    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = token_mint,
        associated_token::authority = vault_signer,
    )]
    pub vault_signer_token_account: InterfaceAccount<'info, TokenAccount>,
    
     #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = depositor,
    )]
    pub depositor_token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub token_program:  Program<'info, Token>,
    
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}

pub fn deposit_token_handler(ctx: Context<DepositToken>, params: DepositTokenParams) -> Result<()> {
    let depositor_account = &ctx.accounts.depositor;
    let depositor_token_account = &ctx.accounts.depositor_token_account;
    let vault_account = &ctx.accounts.vault;
    let vault_signer_account = &ctx.accounts.vault_signer;
    let vault_signer_token_account = &ctx.accounts.vault_signer_token_account;
    let token_program = &ctx.accounts.token_program;
    let token_mint_account = &ctx.accounts.token_mint;

    require_gte!(
        depositor_token_account.amount, 
        params.amount, 
        ErrorCode::NotEnoughTokens
    );

    let cpi_context = CpiContext::new(
        token_program.to_account_info(),
        TransferChecked {
            from: depositor_token_account.to_account_info(),
            to: vault_signer_token_account.to_account_info(),
            authority: depositor_account.to_account_info(),
            mint: token_mint_account.to_account_info(),
        },
    );

    transfer_checked(cpi_context, params.amount, params.decimals)?;

    emit!(TokenDeposited {
        amount: params.amount,
        depositor: depositor_account.key(),
        depositor_token_account: depositor_token_account.key(),
        vault: vault_account.key(),
        vault_signer_token_account: vault_signer_token_account.key(),
        vault_signer: vault_signer_account.key()
    });

    Ok(())
}
