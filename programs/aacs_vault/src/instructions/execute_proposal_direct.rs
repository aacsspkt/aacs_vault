use std::collections::HashSet;

use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};

use crate::{
    constants::VAULT_SIGNER_PREFIX,
    error::ErrorCode,
    events::ProposalDirectExecuted,
    state::{AccountSpec, Action, Vault},
};

use super::ParamAction;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ExecuteProposalDirectParams {
    pub actions: Vec<ParamAction>,
}

#[derive(Accounts)]
#[instruction(params: ExecuteProposalDirectParams)]
pub struct ExecuteProposalDirect<'info> {
    #[account(mut)]
    proposer: Signer<'info>,

    #[account(
        constraint = &vault.owner == proposer.key @ErrorCode::VaultNotOwned
    )]
    vault: Box<Account<'info, Vault>>,

    #[account(
        seeds = [VAULT_SIGNER_PREFIX.as_ref(), vault.key().as_ref()],
        bump = vault.signer_bump,
    )]
    /// CHECK: vault_signer is a PDA program signer. Data is never read or written to
    vault_signer: UncheckedAccount<'info>,

    system_program: Program<'info, System>,
}

pub fn execute_proposal_direct_handler(
    ctx: Context<ExecuteProposalDirect>,
    params: ExecuteProposalDirectParams,
) -> Result<()> {
    let vault_account = &ctx.accounts.vault;
    let vault_signer_account = &ctx.accounts.vault_signer;

    let vault_key = vault_account.key();

    let seeds = &[
        VAULT_SIGNER_PREFIX.as_ref(),
        vault_key.as_ref(),
        &[ctx.accounts.vault.signer_bump],
    ];

    let signer = &[&seeds[..]];

    let actions = params
        .actions
        .iter()
        .map(|acc| {
            let account_specs = acc
                .account_specs
                .iter()
                .map(AccountSpec::from)
                .collect::<Vec<_>>();

            Action::new(acc.program_id, account_specs, acc.data.clone())
        })
        .collect::<Vec<_>>();

    for action in actions.iter() {
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
                if unique_pubkeys.contains(account.key) {
                    Some(account.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        invoke_signed(&ix, &accounts, signer)?;
    }

    emit!(ProposalDirectExecuted {
        actions,
        created_date: Clock::get()?.unix_timestamp,
        vault: vault_account.key()
    });

    Ok(())
}
