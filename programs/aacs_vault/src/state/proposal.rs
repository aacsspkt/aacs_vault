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
    pub fn append_actions(&mut self, actions: &mut Vec<Action>) -> Result<()> {
        self.actions.append(actions);

        Ok(())
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
