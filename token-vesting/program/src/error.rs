use {
    num_derive::FromPrimitive,
    solana_program::{decode_error::DecodeError, program_error::ProgramError},
    thiserror::Error,
};

#[derive(Clone, Debug, Error, FromPrimitive)]
pub enum TokenVestingError {
    #[error("This account is already initialized")]
    AlreadyInitialized,
    #[error("Data type mismatch")]
    DataTypeMismatch,
    #[error("Wrong account owner")]
    WrongOwner,
    #[error("Account is uninitialized")]
    Uninitialized,
    #[error("The provided vault account is invalid")]
    InvalidVaultAccount,
    #[error("Borsh Error")]
    BorshError,
}

impl From<TokenVestingError> for ProgramError {
    fn from(e: TokenVestingError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for TokenVestingError {
    fn type_of() -> &'static str {
        "TokenVestingError"
    }
}
