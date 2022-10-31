//! Create a new token vesting contract

use bonfida_utils::{checks::check_account_owner, BorshSize};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{msg, program::invoke, program_pack::Pack};
use spl_token::state::AccountState;

use crate::{
    error::TokenVestingError,
    state::vesting_contract::{VestingContract, VestingSchedule},
};

use {
    bonfida_utils::{
        checks::{check_account_key, check_signer},
        InstructionsAccount,
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

#[derive(BorshSerialize, BorshDeserialize, BorshSize)]
pub struct Params {
    pub signer_nonce: u8,
    pub schedule: Vec<VestingSchedule>,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// SPL token program account
    pub spl_token_program: &'a T,

    /// The account which will store the [`VestingContract`] data structure
    #[cons(writable)]
    pub vesting_contract: &'a T,

    /// The contract's escrow vault
    #[cons(writable)]
    pub vault: &'a T,

    #[cons(writable)]
    /// The account currently holding the tokens to be vested
    pub source_tokens: &'a T,

    #[cons(signer)]
    /// The owner of the account currently holding the tokens to be vested
    pub source_tokens_owner: &'a T,

    /// The eventual recipient of the vested tokens
    pub recipient: &'a T,
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
            vault: next_account_info(accounts_iter)?,
            source_tokens: next_account_info(accounts_iter)?,
            source_tokens_owner: next_account_info(accounts_iter)?,
            recipient: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.spl_token_program, &spl_token::ID)?;

        // Check owners
        check_account_owner(accounts.vesting_contract, program_id)?;
        check_account_owner(accounts.vault, &spl_token::ID)?;

        // Check signer
        check_signer(accounts.source_tokens_owner)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let Params {
        signer_nonce,
        schedule,
    } = params;

    let expected_vesting_contract_account_size =
        VestingContract::compute_allocation_size(schedule.len());

    if accounts.vesting_contract.data_len() != expected_vesting_contract_account_size {
        msg!("The vesting contract account is incorrectly sized for the supplied schedule!");
        return Err(ProgramError::InvalidArgument);
    }

    check_vault_account(
        accounts.vault,
        program_id,
        *accounts.vesting_contract.key,
        signer_nonce,
    )?;

    let mut vesting_contract_guard = accounts.vesting_contract.data.borrow_mut();

    VestingContract::initialize(&mut vesting_contract_guard)?;
    let vesting_contract = VestingContract {
        owner: *accounts.recipient.key,
        vault: *accounts.vault.key,
        current_schedule_index: 0,
        signer_nonce,
        schedule,
    };

    let mut total_amount = 0u64;
    let mut last_timestamp: u64 = 0;
    for schedule in vesting_contract.schedule.iter() {
        if schedule.unlock_timestamp < last_timestamp {
            msg!("The schedules should be provided in order!");
            return Err(ProgramError::InvalidArgument);
        }
        last_timestamp = schedule.unlock_timestamp;
        total_amount = total_amount.checked_add(schedule.quantity).unwrap();
    }

    let instruction = spl_token::instruction::transfer(
        &spl_token::ID,
        accounts.source_tokens.key,
        accounts.vault.key,
        accounts.source_tokens_owner.key,
        &[],
        total_amount,
    )?;

    invoke(
        &instruction,
        &[
            accounts.spl_token_program.clone(),
            accounts.source_tokens.clone(),
            accounts.vault.clone(),
            accounts.source_tokens_owner.clone(),
        ],
    )?;

    vesting_contract.commit(&mut vesting_contract_guard)?;

    Ok(())
}

fn check_vault_account(
    vault: &AccountInfo,
    program_id: &Pubkey,
    contract_key: Pubkey,
    signer_nonce: u8,
) -> Result<(), ProgramError> {
    let vault_account = spl_token::state::Account::unpack(&vault.data.borrow())?;

    let vault_signer =
        Pubkey::create_program_address(&[&contract_key.to_bytes(), &[signer_nonce]], program_id)?;
    let is_valid = vault_account.owner == vault_signer
        && vault_account.amount == 0
        && vault_account.delegate.is_none()
        && vault_account.state == AccountState::Initialized
        && vault_account.close_authority.is_none();
    if !is_valid {
        return Err(TokenVestingError::InvalidVaultAccount.into());
    }
    Ok(())
}
