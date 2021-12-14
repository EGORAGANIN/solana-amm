#![cfg(feature = "test-bpf")]

use num_traits::FromPrimitive;
use solana_program::decode_error::DecodeError;
use solana_program::instruction::InstructionError;
use solana_program::program_option::COption;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::{Transaction, TransactionError};
use spl_token::state::{Account, AccountState, Mint};
use amm::pda::Pda;
use amm::id;
use amm::entrypoint::process_instruction;

pub struct Env {
    pub ctx: ProgramTestContext,
    pub user_token_x_y_owner_and_payer: Keypair,
    pub minter_x: Keypair,
    pub minter_y: Keypair,
    pub user_token_x_pk: Pubkey,
    pub user_token_y_pk: Pubkey,
}

impl Env {
    const DEPOSIT_AMOUNT: u64 = 5_000_000_000;
    const TOKEN_X_AMOUNT: u64 = 5_000;
    const TOKEN_Y_AMOUNT: u64 = 15_000;

    pub async fn new() -> Env {
        let transfer_program = ProgramTest::new("amm", id(), processor!(process_instruction));
        let mut ctx = transfer_program.start_with_context().await;


        // create test data
        let user_token_x_y_owner_and_payer = Keypair::new();
        let minter_x = Keypair::new();
        let minter_y = Keypair::new();
        let user_token_x = spl_associated_token_account::get_associated_token_address(
            &user_token_x_y_owner_and_payer.pubkey(), &minter_x.pubkey(),
        );
        let user_token_y = spl_associated_token_account::get_associated_token_address(
            &user_token_x_y_owner_and_payer.pubkey(), &minter_y.pubkey(),
        );
        let token_x_decimals = 5;
        let token_y_decimals = 9;


        // deposit lamports
        let user_lamports_before_transfer = ctx.banks_client
            .get_balance(user_token_x_y_owner_and_payer.pubkey())
            .await
            .expect("user_lamports_before_transfer");
        let user_deposit_ix = system_instruction::transfer(
            &ctx.payer.pubkey(),
            &user_token_x_y_owner_and_payer.pubkey(),
            Env::DEPOSIT_AMOUNT,
        );
        let deposit_tx = Transaction::new_signed_with_payer(
            &[user_deposit_ix],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer],
            ctx.last_blockhash,
        );
        ctx.banks_client
            .process_transaction(deposit_tx)
            .await
            .expect("deposit_tx");
        let user_lamports_after_transfer = ctx.banks_client
            .get_balance(user_token_x_y_owner_and_payer.pubkey())
            .await
            .expect("user_lamports_after_transfer");
        assert_eq!(user_lamports_before_transfer, user_lamports_after_transfer - Env::DEPOSIT_AMOUNT);


        // initialize X,Y minters
        Self::initialize_minter(
            &mut ctx,
            &user_token_x_y_owner_and_payer,
            &minter_x,
            &user_token_x_y_owner_and_payer,
            &user_token_x_y_owner_and_payer,
            token_x_decimals,
        ).await;
        Self::initialize_minter(
            &mut ctx,
            &user_token_x_y_owner_and_payer,
            &minter_y,
            &user_token_x_y_owner_and_payer,
            &user_token_x_y_owner_and_payer,
            token_y_decimals,
        ).await;


        // create associated token X,Y accounts
        let user_token_x_pk = spl_associated_token_account::get_associated_token_address(
            &user_token_x_y_owner_and_payer.pubkey(), &minter_x.pubkey(),
        );
        let user_token_x_acc = ctx.banks_client
            .get_account(user_token_x_pk)
            .await
            .expect("user_token_x_acc");
        assert_eq!(user_token_x_acc, None);

        let user_token_y_pk = spl_associated_token_account::get_associated_token_address(
            &user_token_x_y_owner_and_payer.pubkey(), &minter_y.pubkey(),
        );
        let user_token_y_acc = ctx.banks_client
            .get_account(user_token_y_pk)
            .await
            .expect("user_token_y_acc");
        assert_eq!(user_token_y_acc, None);

        let create_user_token_x_ix = spl_associated_token_account::create_associated_token_account(
            &user_token_x_y_owner_and_payer.pubkey(),
            &user_token_x_y_owner_and_payer.pubkey(),
            &minter_x.pubkey(),
        );
        let create_user_token_y_ix = spl_associated_token_account::create_associated_token_account(
            &user_token_x_y_owner_and_payer.pubkey(),
            &user_token_x_y_owner_and_payer.pubkey(),
            &minter_y.pubkey(),
        );
        let create_user_token_x_y_tx = Transaction::new_signed_with_payer(
            &[create_user_token_x_ix, create_user_token_y_ix],
            Some(&user_token_x_y_owner_and_payer.pubkey()),
            &[&user_token_x_y_owner_and_payer],
            ctx.last_blockhash,
        );
        ctx.banks_client
            .process_transaction(create_user_token_x_y_tx)
            .await
            .expect("create_user_token_x_y_tx");

        let user_token_x_acc = ctx.banks_client
            .get_packed_account_data::<Account>(user_token_x_pk)
            .await
            .expect("user_token_x_acc");
        assert_eq!(user_token_x_acc.owner, user_token_x_y_owner_and_payer.pubkey());
        assert_eq!(user_token_x_acc.mint, minter_x.pubkey());
        assert_eq!(user_token_x_acc.amount, 0);
        assert_eq!(user_token_x_acc.state, AccountState::Initialized);

        let user_token_y_acc = ctx.banks_client
            .get_packed_account_data::<Account>(user_token_y_pk)
            .await
            .expect("user_token_x_acc");
        assert_eq!(user_token_y_acc.owner, user_token_x_y_owner_and_payer.pubkey());
        assert_eq!(user_token_y_acc.mint, minter_y.pubkey());
        assert_eq!(user_token_y_acc.amount, 0);
        assert_eq!(user_token_y_acc.state, AccountState::Initialized);


        // mint test tokens
        Self::mint_token(
            &mut ctx,
            &user_token_x_y_owner_and_payer,
            &minter_x,
            &user_token_x,
            &user_token_x_y_owner_and_payer,
            Env::TOKEN_X_AMOUNT,
        ).await;
        Self::mint_token(
            &mut ctx,
            &user_token_x_y_owner_and_payer,
            &minter_y,
            &user_token_y,
            &user_token_x_y_owner_and_payer,
            Env::TOKEN_Y_AMOUNT,
        ).await;

        Env { ctx, user_token_x_y_owner_and_payer, minter_x, minter_y, user_token_x_pk, user_token_y_pk }
    }


    async fn initialize_minter(
        ctx: &mut ProgramTestContext,
        payer: &Keypair,
        minter: &Keypair,
        mint_authority: &Keypair,
        freeze_authority: &Keypair,
        decimals: u8,
    ) {
        let minter_acc_before_init = ctx.banks_client
            .get_account(minter.pubkey())
            .await
            .expect("minter_acc_before_init");
        assert_eq!(minter_acc_before_init, None);

        let rent = ctx.banks_client.get_rent().await.unwrap();
        let mint_rent_value = rent.minimum_balance(Mint::LEN);

        let create_mint_acc_ix = system_instruction::create_account(
            &payer.pubkey(),
            &minter.pubkey(),
            mint_rent_value,
            Mint::LEN as u64,
            &spl_token::id(),
        );
        let init_mint_ix = spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &minter.pubkey(),
            &mint_authority.pubkey(),
            Some(&freeze_authority.pubkey()),
            decimals,
        ).expect("init_mint_ix");
        let init_mint_tx = Transaction::new_signed_with_payer(
            &[create_mint_acc_ix, init_mint_ix],
            Some(&payer.pubkey()),
            &[payer, minter],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(init_mint_tx).await.expect("init_mint_tx");

        let minter_after_init = ctx.banks_client
            .get_packed_account_data::<Mint>(minter.pubkey())
            .await
            .expect("minter_after_init");
        assert_eq!(minter_after_init.mint_authority, COption::Some(mint_authority.pubkey()));
        assert_eq!(minter_after_init.supply, 0);
        assert_eq!(minter_after_init.decimals, decimals);
        assert_eq!(minter_after_init.is_initialized, true);
        assert_eq!(minter_after_init.freeze_authority, COption::Some(freeze_authority.pubkey()));
    }

    async fn mint_token(
        ctx: &mut ProgramTestContext,
        payer: &Keypair,
        minter: &Keypair,
        token_holder: &Pubkey,
        mint_authority: &Keypair,
        amount: u64,
    ) {
        let token_holder_before_mint_to = ctx.banks_client
            .get_packed_account_data::<Account>(*token_holder)
            .await
            .expect("token_holder_before_mint_to");

        let mint_to_ix = spl_token::instruction::mint_to(
            &spl_token::id(),
            &minter.pubkey(),
            &token_holder,
            &mint_authority.pubkey(),
            &[],
            amount,
        ).expect("mint_to_ix");
        let mint_to_tx = Transaction::new_signed_with_payer(
            &[mint_to_ix],
            Some(&payer.pubkey()),
            &[payer, mint_authority],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(mint_to_tx).await.expect("mint_to_tx");

        let token_holder_after_mint_to = ctx.banks_client
            .get_packed_account_data::<Account>(*token_holder)
            .await
            .expect("token_holder_after_mint_to");

        assert_eq!(token_holder_before_mint_to.amount, token_holder_after_mint_to.amount - amount);
    }
}

pub async fn check_pda(ctx: &mut ProgramTestContext, pda: &Pda) {
    let pda_token_x_acc = ctx.banks_client.get_account(pda.pda_token_x_pk)
        .await
        .expect("pda_token_x_acc");
    assert_eq!(pda_token_x_acc, None);

    let pda_token_y_acc = ctx.banks_client.get_account(pda.pda_token_y_pk)
        .await
        .expect("pda_token_y_acc");
    assert_eq!(pda_token_y_acc, None);

    let pda_vault_acc = ctx.banks_client.get_account(pda.vault.0)
        .await
        .expect("pda_vault_acc");
    assert_eq!(pda_vault_acc, None);
}

pub fn decode_error<T: DecodeError<T> + FromPrimitive>(e: TransactionError) -> T {
    match e {
        TransactionError::InstructionError(_, InstructionError::Custom(code)) =>
            T::decode_custom_error_to_enum(code).unwrap(),
        _ => panic!("Unexpected error")
    }
}