use anchor_lang::prelude::*;

use super::AccountSpec;

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Action {
    pub program_id: Pubkey,
    pub account_specs: Vec<AccountSpec>,
    pub data: Vec<u8>,
}

impl Action {
    pub fn new(program_id: Pubkey, account_specs: Vec<AccountSpec>, data: Vec<u8>) -> Self {
        Action {
            program_id,
            account_specs,
            data,
        }
    }
}
