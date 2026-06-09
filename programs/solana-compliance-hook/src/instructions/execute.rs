use anchor_lang::prelude::*;
use anchor_spl::token_interface::{ Mint, TokenAccount };
use crate::state::{ ComplianceConfig, ComplianceList, ComplianceMode };
use crate::error::ComplianceError;

use spl_token_2022::{
    extension::{ transfer_fee::TransferFeeConfig, BaseStateWithExtensions, StateWithExtensions },
    state::Mint as Token2022Mint,
};
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_type_length_value::state::TlvStateBorrowed;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;
use spl_transfer_hook_interface::instruction::TransferHookInstruction;

#[derive(Accounts)]
pub struct ExecuteTransfer<'info> {
    pub source_account: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub destination_account: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: Resolved and guaranteed valid by Token-2022 program context
    pub owner_delegate: AccountInfo<'info>,
    /// CHECK: Structural mapping checked via TLV load methods
    pub extra_metas_account: AccountInfo<'info>,
}

pub fn handler<'info>(ctx: Context<'_, '_, 'info, 'info, ExecuteTransfer<'info>>, _amount: u64)-> Result<()> {
    // 🛡️ Step 1: Validate TLV Registry Integrity
    let extra_metas_info = &ctx.accounts.extra_metas_account;
    let extra_metas_data = extra_metas_info.try_borrow_data()?;

    // 🗲 Parse the TLV state, then extract the extra account meta slice
    let tlv_state = TlvStateBorrowed::unpack(&extra_metas_data).map_err(|_|
        error!(ComplianceError::InvalidTlvData)
    )?;
   let _extra_account_metas = ExtraAccountMetaList::unpack_with_tlv_state::<ExecuteInstruction>(
        &tlv_state
    ).map_err(|_| error!(ComplianceError::InvalidTlvData))?;

  require!(ctx.remaining_accounts.len() >= 2, ComplianceError::InvalidRemainingAccounts);

    // 🗲 Rule: Extract the element out of the slice by reference to maintain the 'info lifespan
    // By matching the exact slot reference inside the slice, we completely avoid local stack variables
    let config_info: &AccountInfo<'info> = &ctx.remaining_accounts[0];
    let list_info: &AccountInfo<'info> = &ctx.remaining_accounts[1];

    // 🖨️ Step 3: Upcast runtime references securely
    // Since config_info and list_info are now pure `&AccountInfo<'info>`, they pass into try_from flawlessly
    let config: Account<ComplianceConfig> = Account::try_from(config_info)?;

    // 🧭 Step 4: Compliance Strategy Engine Routing
    match config.mode {
        ComplianceMode::AllowList => {
            // 🗲 No stack allocation = No drop errors! Passes the 'info lifespan directly down.
            let compliance_account: Account<ComplianceList> = Account::try_from(list_info)?;
            let dest_owner = ctx.accounts.destination_account.owner;

            require!(
                compliance_account.allow_list.contains(&dest_owner),
                ComplianceError::NotAllowlisted
            );
        }
        ComplianceMode::BlackList => {
            let compliance_account: Account<ComplianceList> = Account::try_from(list_info)?;
            let source_owner = ctx.accounts.source_account.owner;
            let dest_owner = ctx.accounts.destination_account.owner;

            require!(
                !compliance_account.block_list.contains(&source_owner),
                ComplianceError::SourceBlacklisted
            );
            require!(
                !compliance_account.block_list.contains(&dest_owner),
                ComplianceError::DestinationBlacklisted
            );
        }
        ComplianceMode::Both => {
            let compliance_account: Account<ComplianceList> = Account::try_from(list_info)?;
            let source_owner = ctx.accounts.source_account.owner;
            let dest_owner = ctx.accounts.destination_account.owner;

            require!(
                compliance_account.allow_list.contains(&dest_owner),
                ComplianceError::NotAllowlisted
            );
            require!(
                !compliance_account.block_list.contains(&source_owner),
                ComplianceError::SourceBlacklisted
            );
            require!(
                !compliance_account.block_list.contains(&dest_owner),
                ComplianceError::DestinationBlacklisted
            );
        }
        ComplianceMode::None => {}
    }

    // 🪙 Step 5: JIT Fee Parity Verification (Bypassing RefCell Deadlocks)
    if config.fee_basis_points > 0 {
        let mint_info = ctx.accounts.mint.to_account_info();
        let mint_data = mint_info.try_borrow_data()?;

        // Safely parse the raw mint bytes across the Token-2022 extension barriers
        let state = StateWithExtensions::<Token2022Mint>
            ::unpack(&mint_data)
            .map_err(|_| error!(ComplianceError::InvalidMintState))?;

        let fee_config = state
            .get_extension::<TransferFeeConfig>()
            .map_err(|_| error!(ComplianceError::MissingFeeExtension))?;

        // Extract current configuration rules from epoch layout metrics
        let current_fee = fee_config.get_epoch_fee(Clock::get()?.epoch);
        let token_fee_bps = u16::from(current_fee.transfer_fee_basis_points);

        // Core Financial Invariant: Token parameters must mirror our configured compliance constraints
        require_eq!(
            token_fee_bps,
            config.fee_basis_points,
            ComplianceError::FeeMismatchedWithExtension
        );
    }

    Ok(())
}

// pub fn fallback<'info>(
//     program_id: &Pubkey,
//     accounts: &'info [AccountInfo<'info>],
//     data: &[u8]
// ) -> Result<()> {
//     let instruction = TransferHookInstruction::unpack(data)?;

//     match instruction {
//         TransferHookInstruction::Execute { amount } => {
//             // Enforce that the slice contains the 5 base accounts + 2 remaining metas
//             if accounts.len() < 7 {
//                 return Err(ProgramError::NotEnoughAccountKeys.into());
//             }

//             // Map indices exactly to the ExecuteTransfer fields
//             let source_account = InterfaceAccount::try_from(&accounts[0])?;
//             let mint = InterfaceAccount::try_from(&accounts[1])?;
//             let destination_account = InterfaceAccount::try_from(&accounts[2])?;
//             let owner_delegate = accounts[3].clone();
//             let extra_metas_account = accounts[4].clone();

//             // Capture trailing compliance metadata accounts (slice reference, not a copy)
//             let remaining_accounts = &accounts[5..];

//             // Bind the structural accounts layout to a named stack variable to prolong lifetime
//             let mut execute_accounts = ExecuteTransfer {
//                 source_account,
//                 mint,
//                 destination_account,
//                 owner_delegate,
//                 extra_metas_account,
//             };

//             // Marshall context with compiler-generated strongly-typed Bumps struct
//             let ctx = Context::new(
//                 program_id,
//                 &mut execute_accounts,
//                 remaining_accounts,
//                 ExecuteTransferBumps
//             );

//             // Direct internal routing to the compliance logic
//             handler(ctx, amount)
//         }
//         _ => Err(ProgramError::InvalidInstructionData.into()),
//     }
// }




pub fn fallback<'info>(
    program_id: &Pubkey,
    accounts: &'info [AccountInfo<'info>], // Already possesses the long 'info lifetime
    data: &[u8]
) -> Result<()> {
    let instruction = TransferHookInstruction::unpack(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        TransferHookInstruction::Execute { amount } => {
            if accounts.len() < 7 {
                return Err(ProgramError::NotEnoughAccountKeys.into());
            }

            let source_account = InterfaceAccount::try_from(&accounts[0])?;
            let mint = InterfaceAccount::try_from(&accounts[1])?;
            let destination_account = InterfaceAccount::try_from(&accounts[2])?;
            let owner_delegate = accounts[3].clone();
            let extra_metas_account = accounts[4].clone();

            // 👑 Fix: Slice directly from the 'info slice reference instead of creating a local Vec
            let remaining_accounts: &'info [AccountInfo<'info>] = &accounts[5..];

            let mut execute_accounts = ExecuteTransfer {
                source_account,
                mint,
                destination_account,
                owner_delegate,
                extra_metas_account,
            };

            let bumps = ExecuteTransferBumps::default();

            // Pass the slice reference directly. Its lifetime matches 'info perfectly.
            let ctx = Context::new(
                program_id,
                &mut execute_accounts,
                remaining_accounts, 
                bumps
            );

            handler(ctx, amount)
        }
        _ => Err(ProgramError::InvalidInstructionData.into())
    }
}
