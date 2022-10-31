use bonfida_utils::BorshSize;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::error::TokenVestingError;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct VestingContract {
    /// The eventual token receiver
    pub owner: Pubkey,
    /// The contract escrow vault
    pub vault: Pubkey,
    /// Index in the current schedule vector of the last completed schedule
    pub current_schedule_index: u64,
    /// Used to generate the signing PDA which owns the vault
    pub signer_nonce: u8,
    /// Describes the token release schedule
    pub schedule: Vec<VestingSchedule>,
}

#[derive(BorshDeserialize, BorshSerialize, BorshSize, Clone)]
/// An item of the vesting schedule
pub struct VestingSchedule {
    /// When the unlock happens as a UTC timestamp
    pub unlock_timestamp: u64,
    /// The quantity of tokens to unlock from the vault
    pub quantity: u64,
}

impl VestingContract {
    /// Initialize a new VestingContract data account
    pub fn initialize(buffer: &mut [u8]) -> Result<(), TokenVestingError> {
        let (tag, _) = buffer.split_at_mut(8);
        let tag: &mut u64 = bytemuck::from_bytes_mut(tag);
        if *tag != super::Tag::Uninitialized as u64 {
            return Err(TokenVestingError::DataTypeMismatch);
        }
        *tag = super::Tag::VestingContract as u64;
        Ok(())
    }

    /// Cast the buffer asa a VestingContract reference wrapper
    pub fn from_buffer(buffer: &[u8], expected_tag: super::Tag) -> Result<Self, TokenVestingError> {
        let (tag, buffer) = buffer.split_at(8);
        if *bytemuck::from_bytes::<u64>(tag) != expected_tag as u64 {
            return Err(TokenVestingError::DataTypeMismatch);
        }
        Self::deserialize(&mut (buffer as &[u8])).map_err(|_| TokenVestingError::BorshError)
    }

    pub fn commit(&self, buffer: &mut [u8]) -> Result<(), TokenVestingError> {
        self.serialize(&mut &mut buffer[8..])
            .map_err(|_| TokenVestingError::BorshError)?;
        Ok(())
    }

    /// Compute a valid allocation size for a VestingContract
    pub fn compute_allocation_size(number_of_schedules: usize) -> usize {
        8 + 32 + 32 + 8 + 1 + 4 + (number_of_schedules * 16)
    }
}
