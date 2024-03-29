use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
};

pub mod vesting_contract;

#[derive(BorshSerialize, BorshDeserialize, BorshSize, PartialEq)]
#[allow(missing_docs)]
pub enum Tag {
    Uninitialized,
    VestingContract,
    ExampleStateBorsh,
}
