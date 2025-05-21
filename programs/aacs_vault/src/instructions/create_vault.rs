use anchor_lang::prelude::*;

use crate::{events::VaultCreated, state::Vault};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateVaultParams {
    pub owner: Pubkey,
    pub signer_bump: u8,
}

#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init, 
        signer, 
        space = 8 + Vault::INIT_SPACE, 
        payer = payer
    )]
    pub vault: Box<Account<'info, Vault>>,

    pub system_program: Program<'info, System>,
}

pub fn create_vault_handler(ctx: Context<CreateVault>, params: CreateVaultParams) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    vault.owner = params.owner;
    vault.signer_bump = params.signer_bump;
    vault.created_date = Clock::get()?.unix_timestamp;

    emit!(VaultCreated {
        vault: vault.key(),
        owner: vault.owner,
        bump: vault.signer_bump,
    });

    Ok(())
}
