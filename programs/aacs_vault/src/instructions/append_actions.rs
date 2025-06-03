use anchor_lang::prelude::*;

use crate::{
    error::ErrorCode,
    events::ActionsAppended,
    state::{AccountSpec, Action, Proposal, Vault},
};

use super::ParamAction;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AppendActionsParams {
    pub actions: Vec<ParamAction>,
    pub proposal_account_size: u32,
}

#[derive(Accounts)]
#[instruction(params: AppendActionsParams)]
pub struct AppendActions<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(
        constraint = &vault.owner == proposer.key @ErrorCode::VaultNotOwned
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut,
        has_one = vault @ErrorCode::ProposalNotOwned,
        constraint = !proposal.is_executed @ErrorCode::ProposalAlreadyExecuted,
        realloc = 8 + params.proposal_account_size as usize,
        realloc::payer = proposer,
        realloc::zero = false,
    )]
    pub proposal: Box<Account<'info, Proposal>>,

    pub system_program: Program<'info, System>,
}

pub fn append_actions_handler(
    ctx: Context<AppendActions>,
    params: AppendActionsParams,
) -> Result<()> {
    let proposal_account = &mut ctx.accounts.proposal;

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

    proposal_account.append_actions(&mut actions.clone());

    emit!(ActionsAppended {
        actions,
        proposal: proposal_account.key(),
    });

    Ok(())
}
