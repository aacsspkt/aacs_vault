use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Error occurred while deriving the pubkey")]
    PubkeyDeriveErrorOccurred,
    #[msg("Cannot access vault not owned by the user")]
    VaultNotOwned,
    #[msg("Not enough lamports for transfer")]
    NotEnoughLamports,
    #[msg("Data allocation space is not sufficient for proposal")]
    InsufficientDataAllocationForProposal,
    #[msg("Proposal does not belongs to vault")]
    ProposalNotOwned,
    #[msg("Proposal is already executed")]
    ProposalAlreadyExecuted,
    #[msg("Proposal is expired")]
    ProposalExpired,
    #[msg("Invalid vault signer")]
    InvalidVaultSigner,
    #[msg("Not enough tokens for lamports")]
    NotEnoughTokens,
}
