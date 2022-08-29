use bonfida_utils::WrappedPodMut;
use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;

use crate::error::TokenVestingError;

#[derive(WrappedPodMut)]
pub struct VestingContract<'a> {
    pub header: &'a mut VestingContractHeader,
    pub schedules: &'a mut [VestingSchedule],
}

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
/// Holds vesting contract metadata
pub struct VestingContractHeader {
    /// The eventual token receiver
    pub owner: Pubkey,
    /// The contract escrow vault
    pub vault: Pubkey,
    /// Index in the current schedule vector of the last completed schedule
    pub current_schedule_index: u64,
    /// Used to generate the signing PDA which owns the vault
    pub signer_nonce: u8,
    pub _padding: [u8; 7],
}

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
/// An item of the vesting schedule
pub struct VestingSchedule {
    /// When the unlock happens as a UTC timestamp
    pub unlock_timestamp: u64,
    /// The quantity of tokens to unlock from the vault
    pub quantity: u64,
}

impl VestingContractHeader {
    pub const LEN: usize = std::mem::size_of::<Self>();
}

impl VestingSchedule {
    pub const LEN: usize = std::mem::size_of::<Self>();
}

impl<'contract> VestingContract<'contract> {
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
    pub fn from_buffer(
        buffer: &'contract mut [u8],
        expected_tag: super::Tag,
    ) -> Result<Self, TokenVestingError> {
        let (tag, buffer) = buffer.split_at_mut(8);
        if *bytemuck::from_bytes_mut::<u64>(tag) != expected_tag as u64 {
            return Err(TokenVestingError::DataTypeMismatch);
        }
        Ok(Self::from_bytes(buffer))
    }

    /// Compute a valid allocation size for a VestingContract
    pub fn compute_allocation_size(number_of_schedules: usize) -> usize {
        number_of_schedules
            .checked_mul(VestingSchedule::LEN)
            .and_then(|n| n.checked_add(VestingContractHeader::LEN))
            .and_then(|n| n.checked_add(8))
            .unwrap()
    }
}
