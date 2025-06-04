use anchor_lang::prelude::*;

use super::Action;

#[account]
pub struct Proposal {
    pub vault: Pubkey,
    pub proposal_stage: ProposalStage,
    pub created_date: i64,
    pub expiry_date: i64,
    pub is_executed: bool,
    pub name: String,
    pub actions: Vec<Action>,
}

impl Proposal {
    pub fn calculate_data_size(name: &str, actions: &[Action]) -> usize {
        let action_data_size: usize = actions.iter().map(|action| action.get_data_size()).sum();
        32 + 1 + 8 + 8 + 1 + 4 + name.len() + 4 + action_data_size
    }
}

#[derive(Default, InitSpace, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub enum ProposalStage {
    #[default]
    Draft = 0,
    Completed = 1,
    Cancelled = 2,
    Failed = 3,
}
