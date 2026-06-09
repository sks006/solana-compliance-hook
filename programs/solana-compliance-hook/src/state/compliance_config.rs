use anchor_lang::prelude::*;
use anchor_lang::InitSpace;
// Bring in the decoupled variant token from the sibling module file
use super::compliance_mode::ComplianceMode;

#[account]
#[derive(InitSpace)]
pub struct ComplianceConfig {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub mode: ComplianceMode,
    pub fee_basis_points: u16,
    pub fee_recipient: Pubkey,
    pub bump: u8,
}

impl ComplianceConfig {
    pub const SEED_PREFIX: &'static [u8] = b"compliance-config";
}
