use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_program::{system_program, sysvar};
use crate::id;
use crate::pda::Pda;

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub enum AmmInstruction {
    /// Initialization of an automated market maker.
    /// Creating and initializing PDA smart contract accounts.
    /// Saving the initial value of the contract tokens X, Y.
    /// X * Y = K
    ///
    /// Accounts expected by this instruction:
    /// 0. `[signer]` - user SPL token X owner
    /// 1. `[signer]` - user SPL token Y owner
    /// 2. `[signer, writable]` - user payer for creating PDA X, Y accounts
    /// 3. `[writable]` - from user SPL token X holder
    /// 4. `[writable]` - from user SPL token Y holder
    /// 5. `[]` - minter SPL token X
    /// 6. `[]` - minter SPL token Y
    /// 7. `[writable]` - contract(PDA) SPL token X holder
    /// 8. `[writable]` - contract(PDA) SPL token Y holder
    /// 9. `[]` - contract(PDA) SPL token X owner
    /// 10. `[]` - contract(PDA) SPL token Y owner
    /// 11. `[writable]` - contract(PDA) Vault
    /// 12. `[]` - Rent sysvar
    /// 13. `[]` - System program
    /// 14. `[]` - SPL Token program
    /// 15. `[]` - SPL associated token account program
    ///
    InitMarket { amount_x: u64, amount_y: u64 },

    /// Swap token with market.
    /// The user add token X(or Y) to contract.
    /// Contract return token Y(or X).
    /// dY = Y - K / (X + dX) / dX = X - K / (Y + dY)
    ///
    /// Accounts expected by this instruction:
    /// 0. `[signer]` - user SPL token owner
    /// 1. `[writable]` - from user SPL token X holder
    /// 2. `[writable]` - from user SPL token Y holder
    /// 3. `[]` - minter SPL token X
    /// 4. `[]` - minter SPL token Y
    /// 5. `[writable]` - contract(PDA) SPL token X holder
    /// 6. `[writable]` - contract(PDA) SPL token Y holder
    /// 7. `[]` - contract(PDA) SPL token X owner
    /// 8. `[]` - contract(PDA) SPL token Y owner
    /// 9. `[writable]` - contract(PDA) Vault
    /// 10. `[]` - SPL token program
    ///
    Swap {
        amount: u64,
        minter_pk: Pubkey,
    },
}

impl AmmInstruction {
    pub fn init_market(
        amount_x: u64,
        amount_y: u64,
        user_owner_token_x_pk: Pubkey,
        user_owner_token_y_pk: Pubkey,
        user_payer_pk: Pubkey,
        user_token_x_pk: Pubkey,
        user_token_y_pk: Pubkey,
        minter_x_pk: Pubkey,
        minter_y_pk: Pubkey,
    ) -> Instruction {
        let mut ix_accounts = vec![
            AccountMeta::new_readonly(user_owner_token_x_pk, true),
            AccountMeta::new_readonly(user_owner_token_y_pk, true),
            AccountMeta::new(user_payer_pk, true),
            AccountMeta::new(user_token_x_pk, false),
            AccountMeta::new(user_token_y_pk, false),
            AccountMeta::new_readonly(minter_x_pk, false),
            AccountMeta::new_readonly(minter_y_pk, false),
        ];
        let pda_accounts = Self::get_pda_account_meta(&minter_x_pk, &minter_y_pk);
        ix_accounts.extend(pda_accounts);
        let program_accounts = vec![
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ];
        ix_accounts.extend(program_accounts);

        Instruction::new_with_borsh(
            id(),
            &AmmInstruction::InitMarket { amount_x, amount_y },
            ix_accounts,
        )
    }

    pub fn swap(
        amount: u64,
        minter_pk: Pubkey,
        user_owner_token_pk: Pubkey,
        user_token_x_pk: Pubkey,
        user_token_y_pk: Pubkey,
        minter_x_pk: Pubkey,
        minter_y_pk: Pubkey,
    ) -> Instruction {
        let mut ix_accounts = vec![
            AccountMeta::new(user_owner_token_pk, true),
            AccountMeta::new(user_token_x_pk, false),
            AccountMeta::new(user_token_y_pk, false),
            AccountMeta::new_readonly(minter_x_pk, false),
            AccountMeta::new_readonly(minter_y_pk, false),
        ];
        let pda_accounts = Self::get_pda_account_meta(&minter_x_pk, &minter_y_pk);
        ix_accounts.extend(pda_accounts);
        let program_accounts = vec![
            AccountMeta::new_readonly(spl_token::id(), false),
        ];
        ix_accounts.extend(program_accounts);

        Instruction::new_with_borsh(
            id(),
            &AmmInstruction::Swap { amount, minter_pk },
            ix_accounts,
        )
    }

    fn get_pda_account_meta(
        minter_x_pk: &Pubkey,
        minter_y_pk: &Pubkey
    ) -> Vec<AccountMeta> {
        let pda = Pda::generate(minter_x_pk, minter_y_pk);
        vec![
            AccountMeta::new(pda.pda_token_x_pk, false),
            AccountMeta::new(pda.pda_token_y_pk, false),
            AccountMeta::new_readonly(pda.pda_owner_token_x.0, false),
            AccountMeta::new_readonly(pda.pda_owner_token_y.0, false),
            AccountMeta::new(pda.vault.0, false),
        ]
    }
}