use {
    num_traits::FromPrimitive,
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
        pubkey::Pubkey,
    },
};

use bonfida_utils::WrappedPod;

use crate::instruction::ProgramInstruction;

pub mod claim;
pub mod create;

pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Beginning processing");
        let instruction = FromPrimitive::from_u8(instruction_data[0])
            .ok_or(ProgramError::InvalidInstructionData)?;
        let instruction_data = &instruction_data[8..];
        msg!("Instruction unpacked");

        match instruction {
            ProgramInstruction::Create => {
                msg!("Instruction: Create");
                let params = create::Params::from_bytes(instruction_data);
                create::process(program_id, accounts, params)?;
            }
            ProgramInstruction::Claim => {
                msg!("Instruction: Claim");
                let params = bytemuck::from_bytes(instruction_data);
                claim::process(program_id, accounts, params)?;
            }
        }

        Ok(())
    }
}
