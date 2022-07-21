use crate::{error::TokenVestingError, processor::Processor};

use {
    num_traits::FromPrimitive,
    solana_program::{
        account_info::AccountInfo, decode_error::DecodeError, entrypoint::ProgramResult, msg,
        program_error::PrintProgramError, pubkey::Pubkey,
    },
};

#[cfg(not(feature = "no-entrypoint"))]
use solana_program::entrypoint;
#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

/// The entrypoint to the program
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Entrypoint");
    if let Err(error) = Processor::process_instruction(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<TokenVestingError>();
        return Err(error);
    }
    Ok(())
}

impl PrintProgramError for TokenVestingError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            TokenVestingError::AlreadyInitialized => {
                msg!("Error: This account is already initialized")
            }
            TokenVestingError::DataTypeMismatch => msg!("Error: Data type mismatch"),
            TokenVestingError::WrongOwner => msg!("Error: Wrong account owner"),
            TokenVestingError::Uninitialized => msg!("Error: Account is uninitialized"),
            TokenVestingError::InvalidVaultAccount => {
                msg!("Error: The provided vault account is invalid")
            }
        }
    }
}
