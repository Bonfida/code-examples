//! Claim unvested tokens

use bonfida_utils::{
    checks::{check_account_key, check_account_owner, check_signer},
    BorshSize,
};
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use solana_program::{clock::Clock, msg, program::invoke_signed, sysvar::Sysvar};

use crate::state::{self, vesting_contract::VestingContract};

use {
    bonfida_utils::InstructionsAccount,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

#[derive(Clone, Copy, Zeroable, Pod, BorshDeserialize, BorshSerialize, BorshSize)]
#[repr(C)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// SPL token program account
    pub spl_token_program: &'a T,

    /// The account which will store the [`VestingContract`] data structure
    #[cons(writable)]
    pub vesting_contract: &'a T,

    /// The signing PDA which owns the vault
    pub vesting_contract_signer: &'a T,

    /// The contract's escrow vault
    #[cons(writable)]
    pub vault: &'a T,

    /// The token account to transfer the unvested assets to
    #[cons(writable)]
    pub destination_token_account: &'a T,

    /// The owner of the current vesting contract
    #[cons(signer)]
    pub owner: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            spl_token_program: next_account_info(accounts_iter)?,
            vesting_contract: next_account_info(accounts_iter)?,
            vesting_contract_signer: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            destination_token_account: next_account_info(accounts_iter)?,
            owner: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.spl_token_program, &spl_token::ID)?;

        // Check owners
        check_account_owner(accounts.vesting_contract, program_id)?;
        check_account_owner(accounts.vault, &spl_token::ID)?;
        check_account_owner(accounts.destination_token_account, &spl_token::ID)?;

        // Check signer
        check_signer(accounts.owner)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], _params: &Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    // We begin by parsing the vesting contract account
    let mut vesting_contract_guard = accounts.vesting_contract.data.borrow_mut();
    let mut vesting_contract =
        VestingContract::from_buffer(&vesting_contract_guard, state::Tag::VestingContract)?;

    // We check that the specified owner actually owns this contract
    if &vesting_contract.owner != accounts.owner.key {
        msg!("Invalid vesting contract owner!");
        return Err(ProgramError::InvalidArgument);
    }

    // We also check that the vault is the correct one
    // Since our vesting contract signer is tied to just one vesting contract
    // This isn't strictly necessary and the call to spl_token would fail.
    // This is defense in depth. Also it makes for nicer error messages.
    if &vesting_contract.vault != accounts.vault.key {
        msg!("Invalid vault provided!");
        return Err(ProgramError::InvalidArgument);
    }

    // We derive and check that the provided contract signer is correct.
    // In the same way, this isn't strictly necessary.
    // The call to invoke_signed would fail if this wasn't the case.
    let contract_signer_key = Pubkey::create_program_address(
        &[
            &accounts.vesting_contract.key.to_bytes(),
            &[vesting_contract.signer_nonce as u8],
        ],
        program_id,
    )?;

    if &contract_signer_key != accounts.vesting_contract_signer.key {
        msg!("Invalid contract signer provided!");
        return Err(ProgramError::InvalidArgument);
    }

    // We get the current timestamp from the Clock sysvar
    let current_timestamp = Clock::get()?.unix_timestamp as u64;

    let mut total_amount_to_transfer: u64 = 0;

    // We saturate the vesting_contract.current_schedule_index variable in case we don't break
    // out of our loop. Not doing this would leave the contract empty but in a weird state
    let current_schedule_index = vesting_contract.current_schedule_index as usize;
    vesting_contract.current_schedule_index = u64::MAX;

    for (idx, s) in vesting_contract.schedule[current_schedule_index..]
        .iter_mut()
        .enumerate()
    {
        if s.unlock_timestamp > current_timestamp {
            // We update the current_schedule_index for the next call to claim
            // This prevents the same quantity from being unlocked twice
            vesting_contract.current_schedule_index = idx as u64;
            break;
        }

        total_amount_to_transfer = total_amount_to_transfer.checked_add(s.quantity).unwrap();
        // We zero out the schedule. This isn't strictly necessary as well since we
        // update the current_schedule_index. Defense in depth.
        s.quantity = 0;
    }

    let transfer_instruction = spl_token::instruction::transfer(
        &spl_token::ID,
        accounts.vault.key,
        accounts.destination_token_account.key,
        accounts.vesting_contract_signer.key,
        &[],
        total_amount_to_transfer,
    )?;

    invoke_signed(
        &transfer_instruction,
        &[
            accounts.spl_token_program.clone(),
            accounts.vault.clone(),
            accounts.destination_token_account.clone(),
            accounts.vesting_contract_signer.clone(),
        ],
        &[&[
            &accounts.vesting_contract.key.to_bytes(),
            &[vesting_contract.signer_nonce as u8],
        ]],
    )?;

    vesting_contract.commit(&mut vesting_contract_guard)?;

    Ok(())
}
