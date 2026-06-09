use anchor_lang::prelude::*;
use crate::state::{ComplianceConfig, ComplianceList, ListType};


#[derive(Accounts)]
#[instruction(list_type: ListType)]
pub struct ManageList<'info> {
    #[account(
        has_one = authority,
        seeds = [ComplianceConfig::SEED_PREFIX, mint.key().as_ref()],
        bump = config.bump
    )]
    pub config: Account<'info, ComplianceConfig>,

    #[account(
        init_if_needed,
        payer = authority,
        space = ComplianceList::MAX_SPACE,
        seeds = [ComplianceList::SEED_PREFIX, mint.key().as_ref()],
        bump
    )]
    pub compliance_list: Account<'info, ComplianceList>,

    /// CHECK: The mint of the token-2022.
    pub mint: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}



pub fn handler_add(ctx: Context<ManageList>, list_type: ListType, target: Pubkey) -> Result<()> {
    // 1. Extract a mutable reference to the internal list structure
    let list= &mut ctx.accounts.compliance_list;

// 2. Enforce logic boundary: list.addresses.len() must be below ComplianceList::MAX_ENTRIES
if list.mint == Pubkey::default() { // Capitalized P
    list.mint = ctx.accounts.mint.key();
    list.bump = ctx.bumps.compliance_list;
}

// Select the vector and its maximum capacity
let (vec, max) = match list_type {
    ListType::Allow => (&mut list.allow_list, 100),
    ListType::Block => (&mut list.block_list, 100),
};

// Pass the precise error variant, not the raw parent enum type
require!(vec.len() < max, crate::error::ComplianceError::ListFull);
    // 3. Conditional: If !list.addresses.contains(&target), push the new target pubkey into the vector
    if !vec.contains(&target){
        vec.push(target);
    }
    msg!("Added {:?} to {:?} list",target,list_type);
    Ok(())

    
}

pub fn handler_remove(ctx: Context<ManageList>, list_type: ListType, target: Pubkey) -> Result<()> {
    // 1. Extract a mutable reference to the internal list structure
    let list= &mut ctx.accounts.compliance_list;


    let vec = match list_type {
        ListType::Allow => &mut list.allow_list,
        ListType::Block => &mut list.block_list,
    };
  

    let pos = vec.iter().position(|&x| x == target)
        .ok_or(crate::error::ComplianceError::AddressNotInList)?;

    vec.swap_remove(pos); // or vec.remove(pos) for order preservation

    msg!("Removed {:?} from {:?} list", target, list_type);
    Ok(())
}

