pub mod error;
pub mod state;
pub mod instructions;

use anchor_lang::prelude::*;
use instructions::*;
use state::*;

declare_id!("9NCPBKj3Xe6WbckqF4m9iEmNcEHeZRcU51Lag3hSztmW");

#[program]
pub mod solana_compliance_hook {
    use super::*;

    pub fn initialize_extra_metas(ctx: Context<InitializeExtraAccountMetas>) -> Result<()> {
        instructions::initialize_extra_metas::handler(ctx)
    }

    pub fn set_mode(
        ctx: Context<SetMode>,
        compliance_mode: ComplianceMode,
        fee_basis_points: u16,
        fee_collector: Pubkey
    ) -> Result<()> {
        instructions::set_mode::handler(ctx, compliance_mode, fee_basis_points, fee_collector)
    }

    pub fn add_to_list(
        ctx: Context<ManageList>,
        list_type: ListType,
        target: Pubkey
    ) -> Result<()> {
        instructions::manage_list::handler_add(ctx, list_type, target)
    }

    pub fn remove_from_list(
        ctx: Context<ManageList>,
        list_type: ListType,
        target: Pubkey
    ) -> Result<()> {
        instructions::manage_list::handler_remove(ctx, list_type, target)
    }

    pub fn fallback<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo<'info>],
        data: &[u8]
    ) -> Result<()> {
        instructions::execute::fallback(program_id, accounts, data)
    }
}
