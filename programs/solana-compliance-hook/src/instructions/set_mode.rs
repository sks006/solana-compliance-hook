use anchor_lang::prelude::*;
use crate::state::{ ComplianceConfig, ComplianceMode };
//  USE THIS INSTEAD (Only where seed initialization is required):

#[derive(Accounts)]
pub struct SetMode<'info> {
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + ComplianceConfig::INIT_SPACE,
        seeds = [ComplianceConfig::SEED_PREFIX, mint.key().as_ref()],
        bump
    )]
    pub config: Account<'info, ComplianceConfig>,

    /// CHECK: The mint of the token-2022.
    pub mint: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<SetMode>,
    compliance_mode: ComplianceMode, // Must match the updated lib.rs identifier token
    fee_basis_points: u16,
    fee_collector: Pubkey
) -> Result<()> {
    require!(fee_basis_points <= 10000, crate::error::ComplianceError::InvalidFeeBps);

    let config = &mut ctx.accounts.config;
    config.authority = ctx.accounts.authority.key();
    config.mint = ctx.accounts.mint.key();
    config.mode = compliance_mode;
    config.fee_basis_points = fee_basis_points;
    config.fee_recipient = fee_collector;
    config.bump = ctx.bumps.config;

    msg!("Compliance config updated. Mode: {:?}, Fee: {} bps", compliance_mode, fee_basis_points);
    Ok(())
}
