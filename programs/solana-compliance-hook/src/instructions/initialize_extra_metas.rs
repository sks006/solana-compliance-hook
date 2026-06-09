use anchor_lang::prelude::*;
// 🗲 Rule: Use anchor_lang's internal re-exports to protect the macro namespace path
use anchor_lang::solana_program::{ program::invoke_signed, system_instruction };

use spl_tlv_account_resolution::{
    account::ExtraAccountMeta,
    state::ExtraAccountMetaList,
    seeds::Seed, // 🗲 Public enum path location
};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

// 🗲 Explicitly inherit your state structures and your error parameters
use crate::state::{ ComplianceConfig, ComplianceList };

#[derive(Accounts)]
pub struct InitializeExtraAccountMetas<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: Must be derived explicitly using the SPL Transfer Hook program namespace
    #[account(mut)]
    pub extra_metas_account: AccountInfo<'info>,
    /// CHECK: Fixed structural dependency
    pub mint: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeExtraAccountMetas>) -> Result<()> {
    // 1. Explicitly trace and verify that extra_metas_account PDA matches:
    //    Seeds: [b"extra-account-metas", mint.key().as_ref()]
    //    Target Program Namespace: crate::id() (our own hook program)

    let extra_metas_pda = Pubkey::find_program_address(
        &[b"extra-account-metas", ctx.accounts.mint.key().as_ref()],
        &crate::id()
    ).0;

    require!(
        ctx.accounts.extra_metas_account.key() == extra_metas_pda,
        crate::error::ComplianceError::InvalidExtraAccountMetaList // optionally add this error
    );

    // 2. Build vector array of three ExtraAccountMeta components:
    //    - Meta 0: Seeds: [b"compliance-config", mint.key().as_ref()] derived from program_id
    //    - Meta 1: Seeds: [b"list", b"allowlist", mint.key().as_ref()] derived from program_id
    //    - Meta 2: Seeds: [b"list", b"blacklist", mint.key().as_ref()] derived from program_id

    let config_meta = ExtraAccountMeta::new_with_seeds(
        &[
            Seed::Literal { bytes: ComplianceConfig::SEED_PREFIX.to_vec() },
            Seed::AccountKey { index: 1 }, // 1 = Mint account index in the transfer
        ],
        false, // not a signer
        false // writable (read-only in execute context)
    ).map_err(|_| error!(crate::error::ComplianceError::InvalidTlvData))?;

    let compliance_list_meta = ExtraAccountMeta::new_with_seeds(
        &[
            Seed::Literal { bytes: ComplianceList::SEED_PREFIX.to_vec() }, // b"compliance-list"
            Seed::AccountKey { index: 1 }, // Leverages the same mint lookup key
        ],
        false,
        false
    ).map_err(|_| error!(crate::error::ComplianceError::InvalidTlvData))?;

    let account_metas = vec![config_meta, compliance_list_meta];

    // 3. Compute execution layout size: ExtraAccountMetaList::size_of(account_metas.len())
    //  Calculate required size and rent
    let data_size: usize = ExtraAccountMetaList::size_of(account_metas.len()).map_err(|_|
        error!(crate::error::ComplianceError::CalculationFailure)
    )?;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(data_size);

    // 4. Invoke system_instruction::create_account CPI using the derived dimensions and runtime rent rules.  Allocate the account (owner = spl_transfer_hook_interface)

    let mint_key = ctx.accounts.mint.key();
    let (_, bump) = Pubkey::find_program_address(
        &[b"extra-account-metas", mint_key.as_ref()],
        &crate::id()
    );

    let signer_seeds: &[&[u8]] = &[b"extra-account-metas", mint_key.as_ref(), &[bump]];

    // 🗲 Provide the cryptographic proof matrix to satisfy the System Program
    invoke_signed(
        &system_instruction::create_account(
            ctx.accounts.payer.key,
            ctx.accounts.extra_metas_account.key,
            lamports,
            data_size as u64,
            &crate::id()
        ),
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.extra_metas_account.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[signer_seeds] // 🗲 This provides the missing signature authority
    )?;

    // 5. Borrow the account data slice mutably: ctx.accounts.extra_metas_account.data.borrow_mut()
    //    Call ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &account_metas)
    // 5. Serialize the list into the account. Serialize the meta list into the account's data buffer
    {
        let mut data = ctx.accounts.extra_metas_account.data.borrow_mut();
        ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &account_metas).map_err(|_|
            error!(crate::error::ComplianceError::InvalidTlvData)
        )?;
    }
    // The mutable borrow ends here, so any future reads are safe.
    Ok(())
}
