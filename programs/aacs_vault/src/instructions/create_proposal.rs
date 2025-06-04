use anchor_lang::prelude::*;

use crate::{
    constants::DEFAULT_FLOW_EXPIRY_DURATION, 
    error::ErrorCode, 
    events::ProposalCreated, 
    state::{AccountSpec, Action, Proposal, ProposalStage, Vault}
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ParamAccountSpec {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl From<&ParamAccountSpec> for AccountSpec {
    fn from(account_spec: &ParamAccountSpec) -> Self {
        AccountSpec {
            pubkey: account_spec.pubkey,
            is_signer: account_spec.is_signer,
            is_writable: account_spec.is_writable,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ParamAction {
    pub program_id: Pubkey,
    pub account_specs: Vec<ParamAccountSpec>,
    pub data: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateProposalParams {
    pub name: String,
    pub actions: Vec<ParamAction>,
    pub proposal_account_size: u32,
}

#[derive(Accounts)]
#[instruction(params: CreateProposalParams)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    
    #[account(
        constraint = &vault.owner == proposer.key @ErrorCode::VaultNotOwned
    )]
    pub vault: Box<Account<'info, Vault>>,
    
    #[account(
        init,
        signer, 
        space = 8 + params.proposal_account_size as usize, 
        payer = proposer
    )]
    pub proposal: Box<Account<'info, Proposal>>,

    pub system_program: Program<'info, System>,
}

pub fn create_proposal_handler(ctx: Context<CreateProposal>, params: CreateProposalParams) -> Result<()> {
    let proposal_account = &mut ctx.accounts.proposal;
    let vault_account = &ctx.accounts.vault;

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

    let required_proposal_size = Proposal::calculate_data_size(&params.name, &actions);
    
    require_gte!(
        params.proposal_account_size as usize, 
        required_proposal_size, 
        ErrorCode::InsufficientDataAllocationForProposal
    );

    proposal_account.name = params.name;
    proposal_account.actions = actions.clone();
    proposal_account.vault = vault_account.key();
    proposal_account.created_date = Clock::get()?.unix_timestamp;
    proposal_account.expiry_date = proposal_account.created_date + DEFAULT_FLOW_EXPIRY_DURATION;
    proposal_account.proposal_stage = ProposalStage::Draft;

    emit!(ProposalCreated {
        actions,
        created_date: proposal_account.created_date,
        name: proposal_account.name.clone(),
        proposal: proposal_account.key(),
        vault: vault_account.key()
    });

    Ok(())
}
