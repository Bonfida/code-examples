//! Example instruction //TODO

use bonfida_utils::{checks::check_account_owner, WrappedPod};
use solana_program::{msg, program::invoke};

use crate::state::{vesting_contract::{VestingContract, VestingSchedule, VestingContractHeader}, Tag, self};

use {
    bonfida_utils::{
        checks::{check_account_key, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
};

#[derive(WrappedPod)]
pub struct Params<'a> {
    signer_nonce: &'a u64,
    schedule: &'a [VestingSchedule]
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

    let Params { signer_nonce, schedule } = params;

    let expected_vesting_contract_account_size = VestingContract::compute_allocation_size(schedule.len());

    if accounts.vesting_contract.data_len() != expected_vesting_contract_account_size {
        msg!("The vesting contract account is incorrectly sized for the supplied schedule!");
        return Err(ProgramError::InvalidArgument)
    }

    let mut vesting_contract_guard = accounts.vesting_contract.data.borrow_mut();

    VestingContract::initialize(&mut vesting_contract_guard)?;
    let vesting_contract = VestingContract::from_buffer(&mut vesting_contract_guard, state::Tag::VestingContract)?;

    *vesting_contract.header = VestingContractHeader { 
        owner: *accounts.recipient.key, 
        vault: *accounts.vault.key,
        current_schedule_index: 0,
        signer_nonce: *signer_nonce as u8,
        _padding: [0;7] 
    };
    
    let mut total_amount = 0u64;
    for (schedule, slot) in schedule.iter().zip(vesting_contract.schedules.iter_mut()) {
        *slot = *schedule;
        total_amount = total_amount.checked_add(schedule.quantity).unwrap();
    }


    let instruction = spl_token::instruction::transfer(
        &spl_token::ID, 
        accounts.source_tokens.key, 
        accounts.vault.key, 
        accounts.source_tokens_owner.key, 
        &[], 
        total_amount
    )?;

    invoke(&instruction, &[
        accounts.spl_token_program.clone(),
        accounts.source_tokens.clone(),
        accounts.vault.clone(),
        accounts.source_tokens_owner.clone()
    ])?;

    // let vault_signer = Pubkey::create_program_address(&[&accounts.vesting_contract.key.to_bytes(), &[*signer_nonce as u8]], program_id)?;


    // Verify the example state account
    // let (example_state_key, _) = VestingContract::find_key(program_id);
    // check_account_key(accounts.example_state_cast, &example_state_key)?;

    // let mut example_state_cast_guard = accounts.example_state_cast.data.borrow_mut();

    // let example_state_cast =
    //     VestingContract::from_buffer(&mut example_state_cast_guard, Tag::VestingContract)?;

    // let mut example_state_borsh_guard = accounts.example_state_borsh.data.borrow_mut();

    // let example_state_borsh =
    //     ExampleStateBorsh::from_buffer(&mut example_state_borsh_guard, Tag::ExampleStateBorsh)?;

    //...

    // Update example state account
    // example_state_borsh.save(&mut example_state_borsh_guard);

    Ok(())
}
