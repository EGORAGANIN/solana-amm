#![cfg(feature = "test-bpf")]

use solana_program::pubkey::Pubkey;
use solana_program_test::ProgramTestContext;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use solana_sdk::transport::TransportError;
use spl_token::error::TokenError;
use spl_token::state::{Account, AccountState};
use amm::error::AmmError;
use amm::instruction::AmmInstruction;
use amm::pda::Pda;
use amm::state::Vault;
use amm::swap::{calc_swap, SwapDirection};
use crate::basic::{check_pda, decode_error, Env};

mod basic;

// Test init market

async fn init_market(
    ctx: &mut ProgramTestContext,
    minter_x: &Keypair,
    minter_y: &Keypair,
    user_token_x_y_owner_and_payer: &Keypair,
    user_token_x_pk: &Pubkey,
    user_token_y_pk: &Pubkey,
    amount_x: u64,
    amount_y: u64,
) -> Result<(), TransportError> {
    let init_ix = AmmInstruction::init_market(
        amount_x,
        amount_y,
        user_token_x_y_owner_and_payer.pubkey(),
        user_token_x_y_owner_and_payer.pubkey(),
        user_token_x_y_owner_and_payer.pubkey(),
        *user_token_x_pk,
        *user_token_y_pk,
        minter_x.pubkey(),
        minter_y.pubkey(),
    );
    let init_tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&user_token_x_y_owner_and_payer.pubkey()),
        &[
            user_token_x_y_owner_and_payer,
            user_token_x_y_owner_and_payer,
            user_token_x_y_owner_and_payer
        ],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(init_tx).await
}

async fn check_init_market(
    ctx: &mut ProgramTestContext,
    minter_x: &Keypair,
    minter_y: &Keypair,
    pda: &Pda,
    amount_x: u64,
    amount_y: u64,
) {
    let pda_token_x_acc_after_init = ctx.banks_client.get_packed_account_data::<Account>(pda.pda_token_x_pk)
        .await
        .expect("pda_token_x_acc_after_init");
    assert_eq!(pda_token_x_acc_after_init.owner, pda.pda_owner_token_x.0);
    assert_eq!(pda_token_x_acc_after_init.state, AccountState::Initialized);
    assert_eq!(pda_token_x_acc_after_init.mint, minter_x.pubkey());
    assert_eq!(pda_token_x_acc_after_init.amount, amount_x);

    let pda_token_t_acc_after_init = ctx.banks_client.get_packed_account_data::<Account>(pda.pda_token_y_pk)
        .await
        .expect("pda_token_y_acc_after_init");
    assert_eq!(pda_token_t_acc_after_init.owner, pda.pda_owner_token_y.0);
    assert_eq!(pda_token_t_acc_after_init.state, AccountState::Initialized);
    assert_eq!(pda_token_t_acc_after_init.mint, minter_y.pubkey());
    assert_eq!(pda_token_t_acc_after_init.amount, amount_y);

    let vault_after_init = ctx.banks_client.get_account_data_with_borsh::<Vault>(pda.vault.0)
        .await
        .expect("vault_after_init");
    assert_eq!(vault_after_init.token_x_amount, amount_x);
    assert_eq!(vault_after_init.token_y_amount, amount_y);
}

#[tokio::test]
async fn init_x_y_market() {
    let mut env = Env::new().await;
    let ctx = &mut env.ctx;
    let amount_x = 100;
    let amount_y = 300;

    let pda = Pda::generate(&env.minter_x.pubkey(), &env.minter_y.pubkey());
    check_pda(ctx, &pda).await;

    init_market(
        ctx,
        &env.minter_x,
        &env.minter_y,
        &env.user_token_x_y_owner_and_payer,
        &env.user_token_x_pk,
        &env.user_token_y_pk,
        amount_x,
        amount_y,
    ).await.expect("init_market");
    check_init_market(ctx, &env.minter_x, &env.minter_y, &pda, amount_x, amount_y).await;
}

#[tokio::test]
async fn init_market_unknown_minter() {
    let mut env = Env::new().await;
    let ctx = &mut env.ctx;
    let amount_x = 100;
    let amount_y = 300;
    let unknown_minter_x = Keypair::new();

    let pda = Pda::generate(&env.minter_x.pubkey(), &env.minter_y.pubkey());
    check_pda(ctx, &pda).await;

    let init_error = init_market(
        ctx,
        &unknown_minter_x,
        &env.minter_y,
        &env.user_token_x_y_owner_and_payer,
        &env.user_token_x_pk,
        &env.user_token_y_pk,
        amount_x,
        amount_y,
    ).await
        .expect_err("init_error")
        .unwrap();

    assert_eq!(
        decode_error::<TokenError>(init_error),
        TokenError::InvalidMint
    );
}

#[tokio::test]
async fn init_market_same_minter() {
    let mut env = Env::new().await;
    let ctx = &mut env.ctx;
    let amount_x = 100;
    let amount_y = 300;
    let same_minter = &env.minter_x;

    let pda = Pda::generate(&env.minter_x.pubkey(), &env.minter_y.pubkey());
    check_pda(ctx, &pda).await;

    let init_error = init_market(
        ctx,
        same_minter,
        same_minter,
        &env.user_token_x_y_owner_and_payer,
        &env.user_token_x_pk,
        &env.user_token_y_pk,
        amount_x,
        amount_y,
    ).await
        .expect_err("init_error")
        .unwrap();

    assert_eq!(
        decode_error::<AmmError>(init_error),
        AmmError::IdenticalMinter
    );
}

#[tokio::test]
async fn init_market_zero_amount() {
    let mut env = Env::new().await;
    let ctx = &mut env.ctx;
    let amount_x = 0;
    let amount_y = 0;

    let pda = Pda::generate(&env.minter_x.pubkey(), &env.minter_y.pubkey());
    check_pda(ctx, &pda).await;

    let init_error = init_market(
        ctx,
        &env.minter_x,
        &env.minter_y,
        &env.user_token_x_y_owner_and_payer,
        &env.user_token_x_pk,
        &env.user_token_y_pk,
        amount_x,
        amount_y,
    ).await
        .expect_err("init_error")
        .unwrap();

    assert_eq!(
        decode_error::<AmmError>(init_error),
        AmmError::AmountZero
    );
}


// Test swap

async fn swap(
    ctx: &mut ProgramTestContext,
    minter_x: &Keypair,
    minter_y: &Keypair,
    user_token_x_y_owner_and_payer: &Keypair,
    user_token_x_pk: &Pubkey,
    user_token_y_pk: &Pubkey,
    pda: &Pda,
    swap: &Pubkey,
    amount: u64,
) {
    let swap_direction = SwapDirection::new(swap, &minter_x.pubkey(), &minter_y.pubkey()).expect("swap_direction");

    let pda_token_x_acc_before_swap = ctx.banks_client
        .get_packed_account_data::<Account>(pda.pda_token_x_pk)
        .await
        .expect("pda_token_x_acc_before_swap");
    let pda_token_y_acc_before_swap = ctx.banks_client
        .get_packed_account_data::<Account>(pda.pda_token_y_pk)
        .await
        .expect("pda_token_y_acc_before_swap");
    let invariant_before_swap = pda_token_x_acc_before_swap.amount
        .checked_mul(pda_token_y_acc_before_swap.amount)
        .expect("invariant_before_swap");
    let vault_before_swap = ctx.banks_client.get_account_data_with_borsh::<Vault>(pda.vault.0)
        .await
        .expect("vault_before_swap");
    let invariant_vault_before_swap = vault_before_swap.token_x_amount
        .checked_mul(vault_before_swap.token_y_amount)
        .expect("invariant_vault_before_swap");
    let (take_user_token_pk, return_pda_token_pk) = match swap_direction {
        SwapDirection::XtoY => (user_token_x_pk, pda.pda_token_y_pk),
        SwapDirection::YtoX => (user_token_y_pk, pda.pda_token_x_pk)
    };
    let take_user_token_acc_before_swap = ctx.banks_client
        .get_packed_account_data::<Account>(*take_user_token_pk)
        .await
        .expect("take_user_token_acc_before_swap");
    let return_pda_token_acc_before_swap = ctx.banks_client
        .get_packed_account_data::<Account>(return_pda_token_pk)
        .await
        .expect("return_pda_token_acc_before_swap");

    let swap_ix = AmmInstruction::swap(
        amount,
        *swap,
        user_token_x_y_owner_and_payer.pubkey(),
        *user_token_x_pk,
        *user_token_y_pk,
        minter_x.pubkey(),
        minter_y.pubkey(),
    );
    let swap_tx = Transaction::new_signed_with_payer(
        &[swap_ix],
        Some(&user_token_x_y_owner_and_payer.pubkey()),
        &[user_token_x_y_owner_and_payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(swap_tx).await.expect("swap_tx");

    let pda_token_x_acc_after_swap = ctx.banks_client
        .get_packed_account_data::<Account>(pda.pda_token_x_pk)
        .await
        .expect("pda_token_x_acc_after_swap");
    let pda_token_y_acc_after_swap = ctx.banks_client
        .get_packed_account_data::<Account>(pda.pda_token_y_pk)
        .await
        .expect("pda_token_y_acc_after_swap");
    let invariant_after_swap = pda_token_x_acc_after_swap.amount
        .checked_mul(pda_token_y_acc_after_swap.amount)
        .expect("invariant_after_swap");
    let vault_after_swap = ctx.banks_client.get_account_data_with_borsh::<Vault>(pda.vault.0)
        .await
        .expect("vault_after_swap");
    let invariant_vault_after_swap = vault_after_swap.token_x_amount
        .checked_mul(vault_after_swap.token_y_amount)
        .expect("invariant_vault_after_swap");
    let take_user_token_acc_after_swap = ctx.banks_client
        .get_packed_account_data::<Account>(*take_user_token_pk)
        .await
        .expect("take_user_token_acc_after_swap");
    let return_pda_token_acc_after_swap = ctx.banks_client
        .get_packed_account_data::<Account>(return_pda_token_pk)
        .await
        .expect("return_pda_token_acc_after_swap");

    assert_eq!(invariant_before_swap, invariant_vault_before_swap);
    assert_eq!(invariant_before_swap, invariant_after_swap);
    assert_eq!(invariant_after_swap, invariant_vault_after_swap);

    let swap_result = match swap_direction {
        SwapDirection::XtoY => calc_swap(
            amount,
            pda_token_x_acc_before_swap.amount,
            pda_token_y_acc_before_swap.amount,
        ),
        SwapDirection::YtoX => calc_swap(
            amount,
            pda_token_y_acc_before_swap.amount,
            pda_token_x_acc_before_swap.amount,
        )
    }.expect("swap_result");

    assert_eq!(
        take_user_token_acc_before_swap.amount,
        take_user_token_acc_after_swap.amount + swap_result.take_amount
    );
    assert_eq!(
        return_pda_token_acc_before_swap.amount,
        return_pda_token_acc_after_swap.amount + swap_result.return_amount
    );
}


#[tokio::test]
async fn swap_x_to_y() {
    let mut env = Env::new().await;
    let ctx = &mut env.ctx;
    let amount_x = 500;
    let amount_y = 300;
    let swap_pk = &env.minter_x.pubkey();

    let pda = Pda::generate(&env.minter_x.pubkey(), &env.minter_y.pubkey());
    check_pda(ctx, &pda).await;

    init_market(
        ctx,
        &env.minter_x,
        &env.minter_y,
        &env.user_token_x_y_owner_and_payer,
        &env.user_token_x_pk,
        &env.user_token_y_pk,
        amount_x,
        amount_y,
    ).await.expect("init_market");
    check_init_market(ctx, &env.minter_x, &env.minter_y, &pda, amount_x, amount_y).await;

    swap(
        ctx,
        &env.minter_x,
        &env.minter_y,
        &env.user_token_x_y_owner_and_payer,
        &env.user_token_x_pk,
        &env.user_token_y_pk,
        &pda,
        swap_pk,
        100,
    ).await;
}


#[tokio::test]
async fn swap_y_to_x() {
    let mut env = Env::new().await;
    let ctx = &mut env.ctx;
    let amount_x = 500;
    let amount_y = 300;
    let swap_pk = &env.minter_y.pubkey();

    let pda = Pda::generate(&env.minter_x.pubkey(), &env.minter_y.pubkey());
    check_pda(ctx, &pda).await;

    init_market(
        ctx,
        &env.minter_x,
        &env.minter_y,
        &env.user_token_x_y_owner_and_payer,
        &env.user_token_x_pk,
        &env.user_token_y_pk,
        amount_x,
        amount_y,
    ).await.expect("init_market");
    check_init_market(ctx, &env.minter_x, &env.minter_y, &pda, amount_x, amount_y).await;

    swap(
        ctx,
        &env.minter_x,
        &env.minter_y,
        &env.user_token_x_y_owner_and_payer,
        &env.user_token_x_pk,
        &env.user_token_y_pk,
        &pda,
        swap_pk,
        100,
    ).await;
}

#[tokio::test]
async fn swap_x_to_y_revert_amount() {
    let mut env = Env::new().await;
    let ctx = &mut env.ctx;
    let amount_x = 300;
    let amount_y = 500;
    let swap_pk = &env.minter_x.pubkey();

    let pda = Pda::generate(&env.minter_x.pubkey(), &env.minter_y.pubkey());
    check_pda(ctx, &pda).await;

    init_market(
        ctx,
        &env.minter_x,
        &env.minter_y,
        &env.user_token_x_y_owner_and_payer,
        &env.user_token_x_pk,
        &env.user_token_y_pk,
        amount_x,
        amount_y,
    ).await.expect("init_market");
    check_init_market(ctx, &env.minter_x, &env.minter_y, &pda, amount_x, amount_y).await;

    swap(
        ctx,
        &env.minter_x,
        &env.minter_y,
        &env.user_token_x_y_owner_and_payer,
        &env.user_token_x_pk,
        &env.user_token_y_pk,
        &pda,
        swap_pk,
        100,
    ).await;
}

#[tokio::test]
async fn swap_y_to_x_revert_amount() {
    let mut env = Env::new().await;
    let ctx = &mut env.ctx;
    let amount_x = 300;
    let amount_y = 500;
    let swap_pk = &env.minter_y.pubkey();

    let pda = Pda::generate(&env.minter_x.pubkey(), &env.minter_y.pubkey());
    check_pda(ctx, &pda).await;

    init_market(
        ctx,
        &env.minter_x,
        &env.minter_y,
        &env.user_token_x_y_owner_and_payer,
        &env.user_token_x_pk,
        &env.user_token_y_pk,
        amount_x,
        amount_y,
    ).await.expect("init_market");
    check_init_market(ctx, &env.minter_x, &env.minter_y, &pda, amount_x, amount_y).await;

    swap(
        ctx,
        &env.minter_x,
        &env.minter_y,
        &env.user_token_x_y_owner_and_payer,
        &env.user_token_x_pk,
        &env.user_token_y_pk,
        &pda,
        swap_pk,
        100,
    ).await;
}

#[tokio::test]
async fn swap_without_inited_market() {
    let mut env = Env::new().await;
    let ctx = &mut env.ctx;
    let swap_pk = env.minter_y.pubkey();
    let amount = 100;

    let pda = Pda::generate(&env.minter_x.pubkey(), &env.minter_y.pubkey());
    check_pda(ctx, &pda).await;

    // swap
    let swap_ix = AmmInstruction::swap(
        amount,
        swap_pk,
        env.user_token_x_y_owner_and_payer.pubkey(),
        env.user_token_x_pk,
        env.user_token_y_pk,
        env.minter_x.pubkey(),
        env.minter_y.pubkey(),
    );
    let swap_tx = Transaction::new_signed_with_payer(
        &[swap_ix],
        Some(&env.user_token_x_y_owner_and_payer.pubkey()),
        &[&env.user_token_x_y_owner_and_payer],
        ctx.last_blockhash,
    );
    let swap_error = ctx.banks_client.process_transaction(swap_tx).await
        .expect_err("swap_error")
        .unwrap();

    assert_eq!(
        decode_error::<AmmError>(swap_error),
        AmmError::InvalidVault
    );
}

#[tokio::test]
async fn swap_zero_amount() {
    let mut env = Env::new().await;
    let ctx = &mut env.ctx;
    let swap_pk = env.minter_y.pubkey();
    let amount = 0;

    let pda = Pda::generate(&env.minter_x.pubkey(), &env.minter_y.pubkey());
    check_pda(ctx, &pda).await;

    // swap
    let swap_ix = AmmInstruction::swap(
        amount,
        swap_pk,
        env.user_token_x_y_owner_and_payer.pubkey(),
        env.user_token_x_pk,
        env.user_token_y_pk,
        env.minter_x.pubkey(),
        env.minter_y.pubkey(),
    );
    let swap_tx = Transaction::new_signed_with_payer(
        &[swap_ix],
        Some(&env.user_token_x_y_owner_and_payer.pubkey()),
        &[&env.user_token_x_y_owner_and_payer],
        ctx.last_blockhash,
    );
    let swap_error = ctx.banks_client.process_transaction(swap_tx).await
        .expect_err("swap_error")
        .unwrap();

    assert_eq!(
        decode_error::<AmmError>(swap_error),
        AmmError::AmountZero
    );
}

