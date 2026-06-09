use anchor_lang::prelude::*;

// 🗲 Rule: Maintain InitSpace derivation to ensure the parent state account can dynamically sum size dimensions
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum ComplianceMode {
    None,
    AllowList,
    BlackList,
    Both,
}