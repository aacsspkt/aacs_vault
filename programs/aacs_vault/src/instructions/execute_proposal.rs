use std::collections::HashSet;

use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};

use crate::{
    constants::VAULT_SIGNER_PREFIX,
    error::ErrorCode,
    state::{Proposal, ProposalStage, Vault},
};

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    pub caller: Signer<'info>,

    #[account(
        mut, 
        has_one = vault @ErrorCode::InvalidVault,
        constraint = !proposal.is_executed @ErrorCode::ProposalAlreadyExecuted
    )]
    pub proposal: Box<Account<'info, Proposal>>,

    #[account(
        constraint = &vault.owner == caller.key @ErrorCode::NotVaultOwner
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        seeds = [VAULT_SIGNER_PREFIX.as_ref(), vault.key().as_ref()],
        bump = vault.signer_bump,
    )]
    /// CHECK: vault_signer is a PDA program signer. Data is never read or written to
    pub vault_signer: UncheckedAccount<'info>,
}

pub fn execute_proposal_handler(ctx: Context<ExecuteProposal>) -> Result<()> {
    let vault_account = &ctx.accounts.vault;
    let vault_signer_account = &ctx.accounts.vault_signer;
    let proposal_account = &mut ctx.accounts.proposal;

    let now = Clock::get()?.unix_timestamp;
    require_gte!(
        proposal_account.expiry_date,
        now,
        ErrorCode::ProposalExpired
    );

    let vault_key = vault_account.key();
    let seeds = &[
        VAULT_SIGNER_PREFIX.as_ref(),
        vault_key.as_ref(),
        &[ctx.accounts.vault.signer_bump],
    ];
    let signer = &[&seeds[..]];

    for action in proposal_account.actions.iter() {
        let mut unique_pubkeys: HashSet<Pubkey> = HashSet::new();
        
        unique_pubkeys.insert(action.program_id);

        let account_metas = action
            .account_specs
            .iter()
            .map(|account_spec| {
                unique_pubkeys.insert(account_spec.pubkey);

                let mut account_meta = AccountMeta::from(account_spec);

                if &account_meta.pubkey == vault_signer_account.key {
                    account_meta.is_signer = true;
                }

                account_meta
            })
            .collect::<Vec<_>>();

        let ix = Instruction {
            accounts: account_metas,
            data: action.data.clone(),
            program_id: action.program_id,
        };

        let accounts = ctx
            .remaining_accounts
            .iter()
            .filter_map(|account| {
                if unique_pubkeys.contains(account.key)  {
                    Some(account.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();      

        let result = invoke_signed(&ix, &accounts, signer);

        if result.is_err() {
            proposal_account.proposal_stage = ProposalStage::Failed;

            return Err(result.unwrap_err().into());
        }
    }

    // Burn the transaction to ensure one time use.
    proposal_account.is_executed = true;
    proposal_account.proposal_stage = ProposalStage::Completed;

    Ok(())
}
