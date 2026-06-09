use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum ListType {
    Allow,
    Block,
}

#[account]
#[derive(InitSpace)]
pub struct ComplianceList {
    pub mint: Pubkey,
    #[max_len(100)]
    pub allow_list: Vec<Pubkey>,
    #[max_len(100)]
    pub block_list: Vec<Pubkey>,
    pub bump: u8,
}

impl ComplianceList {
    pub const SEED_PREFIX: &'static [u8] = b"compliance-list";
    // Bind your maximum footprint token to the derived initialization space
    pub const MAX_SPACE: usize = 8 + <Self as Space>::INIT_SPACE;
}