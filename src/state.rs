use borsh::BorshSerialize;
use borsh::BorshDeserialize;

/// Vault of balances of X, Y tokens of the market.
/// Unique for every different X, Y tokens.
/// Needed because an attacker can add tokens in PDA of
/// a Solana on-chain program for violate the ratio X * Y = K
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub struct Vault {
    pub token_x_amount: u64,
    pub token_y_amount: u64
}