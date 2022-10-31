pub use crate::processor::claim;
pub use crate::processor::create;
use {
    bonfida_utils::InstructionsAccount,
    borsh::{BorshDeserialize, BorshSerialize},
    num_derive::FromPrimitive,
    solana_program::{instruction::Instruction, pubkey::Pubkey},
};
#[allow(missing_docs)]
#[derive(BorshDeserialize, BorshSerialize, FromPrimitive)]
pub enum ProgramInstruction {
    /// An example instruction //TODO
    ///
    /// | Index | Writable | Signer | Description                   |
    /// | --------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account    |
    /// | 1     | ❌        | ❌      | The SPL token program account |
    /// | 2     | ✅        | ✅      | Fee payer account             |
    Create,
    Claim,
}
#[allow(missing_docs)]
pub fn create(accounts: create::Accounts<Pubkey>, params: create::Params) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::Create as u8, params)
}
#[allow(missing_docs)]
pub fn claim(accounts: claim::Accounts<Pubkey>, params: claim::Params) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::Claim as u8, params)
}
