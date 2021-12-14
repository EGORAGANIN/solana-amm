use solana_program::pubkey::Pubkey;
use crate::id;

pub const SPL_TOKEN_X_OWNER_SEED: &[u8] = b"SPL_TOKEN_X_OWNER";
pub const SPL_TOKEN_Y_OWNER_SEED: &[u8] = b"SPL_TOKEN_Y_OWNER";
pub const VAULT_SEED: &[u8] = b"VAULT";

#[derive(Debug, Clone)]
pub struct Pda {
    pub pda_owner_token_x: (Pubkey, u8),
    pub pda_owner_token_y: (Pubkey, u8),
    pub pda_token_x_pk: Pubkey,
    pub pda_token_y_pk: Pubkey,
    pub vault: (Pubkey, u8),
}

impl Pda {
    pub fn generate(minter_x_pk: &Pubkey, minter_y_pk: &Pubkey) -> Pda {
        let pda_owner_token_x = find_pk_and_bump(
            SPL_TOKEN_X_OWNER_SEED, minter_x_pk, minter_y_pk,
        );
        let pda_token_x_pk = spl_associated_token_account::get_associated_token_address(
            &pda_owner_token_x.0,
            minter_x_pk,
        );

        let pda_owner_token_y = find_pk_and_bump(
            SPL_TOKEN_Y_OWNER_SEED, minter_x_pk, minter_y_pk,
        );
        let pda_token_y_pk = spl_associated_token_account::get_associated_token_address(
            &pda_owner_token_y.0,
            minter_y_pk,
        );

        let vault = find_pk_and_bump(
            VAULT_SEED, minter_x_pk, minter_y_pk,
        );

        Pda { pda_owner_token_x, pda_owner_token_y, pda_token_x_pk, pda_token_y_pk, vault }
    }
}

pub fn find_pk_and_bump(
    key_name: &[u8],
    minter_x: &Pubkey,
    minter_y: &Pubkey
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            key_name,
            &minter_x.to_bytes(),
            &minter_y.to_bytes(),
            &spl_token::id().to_bytes(),
        ],
        &id()
    )
}
