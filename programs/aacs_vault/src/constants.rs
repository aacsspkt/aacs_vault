use anchor_lang::prelude::*;

#[constant]
pub const VAULT_SIGNER_PREFIX: &[u8; 17] = b"aacs_vault_signer";

pub const DEFAULT_FLOW_EXPIRY_DURATION: i64 = 30 * 24 * 60 * 60; // 1 month
