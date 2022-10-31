#![cfg(feature = "benchmarking")]
use bonfida_test_utils::{ProgramTestContextExt, ProgramTestExt};
use bonfida_utils::bench::get_env_arg;
use borsh::BorshSerialize;
use solana_program::pubkey::Pubkey;
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{account::Account, signature::Keypair, signer::Signer};
use std::cell::RefCell;
use token_vesting::{
    entrypoint::process_instruction,
    state::vesting_contract::{VestingContract, VestingSchedule},
};
pub mod common;
use crate::common::utils::sign_send_instructions;

#[tokio::test]
async fn main() {
    run().await;
}

async fn run() {
    let schedule_length = get_env_arg(0).unwrap_or(100);
    // Create program and test environment
    const ALICE: usize = 0;
    const BOB: usize = 1;
    const MINT_AUTHORITY: usize = 2;

    const SECONDS_IN_HOUR: u64 = 3600;

    let keypairs = [Keypair::new(), Keypair::new(), Keypair::new()];

    let mut program_test = ProgramTest::new(
        "token_vesting",
        token_vesting::ID,
        processor!(process_instruction),
    );

    let (mint_key, _) = program_test.add_mint(None, 6, &keypairs[MINT_AUTHORITY].pubkey());

    ////
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;

    // Initialize Alice and Bob's token accounts:
    let ata_keys = prg_test_ctx
        .initialize_token_accounts(
            mint_key,
            &keypairs[0..2]
                .iter()
                .map(|k| k.pubkey())
                .collect::<Vec<_>>(),
        )
        .await
        .unwrap();

    // Alice vests 16 tokens for Bob
    // We first define the schedule we want

    let now = prg_test_ctx.get_current_timestamp().await.unwrap() as u64;

    let schedule = (0..schedule_length)
        .map(|i| VestingSchedule {
            unlock_timestamp: now + (i + 1) * 3600,
            quantity: 1_000_000 * ((i % 10) + 1),
        })
        .collect::<Vec<_>>();
    let total_number_of_tokens = schedule.iter().map(|s| s.quantity).sum::<u64>();
    // Alice gets 100 tokens
    prg_test_ctx
        .mint_tokens(
            &keypairs[MINT_AUTHORITY],
            &mint_key,
            &ata_keys[ALICE],
            total_number_of_tokens,
        )
        .await
        .unwrap();

    // We then need to allocate our vesting contract account
    // The first step is to find the size to allocate

    let allocation_size = VestingContract::compute_allocation_size(schedule.len());
    let vesting_contract = prg_test_ctx
        .initialize_new_account(allocation_size, token_vesting::ID)
        .await
        .unwrap();

    let (vault_signer, vault_signer_nonce) =
        Pubkey::find_program_address(&[&vesting_contract.to_bytes()], &token_vesting::ID);
    let vault = prg_test_ctx
        .initialize_token_accounts(mint_key, &[vault_signer])
        .await
        .unwrap()[0];

    // We then create the vesting contract
    let ix = token_vesting::instruction::create(
        token_vesting::instruction::create::Accounts {
            spl_token_program: &spl_token::ID,
            vesting_contract: &vesting_contract,
            vault: &vault,
            source_tokens: &ata_keys[ALICE],
            source_tokens_owner: &keypairs[ALICE].pubkey(),
            recipient: &keypairs[BOB].pubkey(),
        },
        token_vesting::instruction::create::Params {
            signer_nonce: &(vault_signer_nonce as u64),
            schedule: &schedule,
        },
    );

    prg_test_ctx
        .sign_send_instructions(&[ix], &[&keypairs[ALICE]])
        .await
        .unwrap();

    let alice_token_account_balance = prg_test_ctx
        .get_token_account(ata_keys[ALICE])
        .await
        .unwrap()
        .amount;

    // Let's claim the schedules one by one

    for v in schedule.into_iter().take(1) {
        // We fast-forward to the unlock
        // let previous_balance = prg_test_ctx
        //     .get_token_account(ata_keys[BOB])
        //     .await
        //     .unwrap()
        //     .amount;
        prg_test_ctx
            .warp_to_timestamp(v.unlock_timestamp as i64)
            .await
            .unwrap();
        let ix = token_vesting::instruction::claim(
            token_vesting::instruction::claim::Accounts {
                spl_token_program: &spl_token::ID,
                vesting_contract: &vesting_contract,
                vesting_contract_signer: &vault_signer,
                vault: &vault,
                destination_token_account: &ata_keys[BOB],
                owner: &keypairs[BOB].pubkey(),
            },
            token_vesting::instruction::claim::Params {},
        );

        prg_test_ctx
            .sign_send_instructions(&[ix], &[&keypairs[BOB]])
            .await
            .unwrap();

        // // We check that the tokens have been properly unvested
        // let bob_token_account_balance = prg_test_ctx
        //     .get_token_account(ata_keys[BOB])
        //     .await
        //     .unwrap()
        //     .amount;
    }
}
