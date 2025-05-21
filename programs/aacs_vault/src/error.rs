use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Error occurred while deriving the pubkey")]
    PubkeyDeriveErrorOccurred,
    #[msg("Cannot access vault not owned by the user")]
    NotVaultOwner,
    #[msg("Not enough lamports for transfer")]
    NotEnoughLamports,
    #[msg("Proposal does not belongs to vault")]
    InvalidVault,
    #[msg("Proposal is already executed")]
    ProposalAlreadyExecuted,
    #[msg("Proposal is expired")]
    ProposalExpired,
    #[msg("Invalid vault signer")]
    InvalidVaultSigner,
}
