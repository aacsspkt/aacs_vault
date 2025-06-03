use anchor_lang::prelude::*;

use crate::state::Action;

#[event]
pub struct VaultCreated {
    pub vault: Pubkey,
    pub owner: Pubkey,
    pub bump: u8,
}

#[event]
pub struct SolDeposited {
    pub vault: Pubkey,
    pub vault_signer: Pubkey,
    pub depositor: Pubkey,
    pub amount: u64,
}

#[event]
pub struct SolWithdrawn {
    pub vault: Pubkey,
    pub withdrawer: Pubkey,
    pub amount: u64,
}

#[event]
pub struct TokenDeposited {
    pub vault: Pubkey,
    pub vault_signer: Pubkey,
    pub vault_signer_token_account: Pubkey,
    pub depositor: Pubkey,
    pub depositor_token_account: Pubkey,
    pub amount: u64,
}

#[event]
pub struct TokenWithdrawn {
    pub vault: Pubkey,
    pub vault_signer: Pubkey,
    pub vault_signer_token_account: Pubkey,
    pub withdrawer: Pubkey,
    pub withdrawer_token_account: Pubkey,
    pub amount: u64,
}

#[event]
pub struct ProposalCreated {
    pub proposal: Pubkey,
    pub vault: Pubkey,
    pub name: String,
    pub created_date: i64,
    pub actions: Vec<Action>,
}

#[event]
pub struct ActionsAppended {
    pub proposal: Pubkey,
    pub actions: Vec<Action>,
}

#[event]
pub struct ProposalExecuted {
    pub vault: Pubkey,
    pub proposal: Pubkey,
    pub executed_date: i64,
}

#[event]
pub struct ProposalDirectExecuted {
    pub vault: Pubkey,
    pub created_date: i64,
    pub actions: Vec<Action>,
}
