use solana_program::decode_error::DecodeError;
use solana_program::program_error::{PrintProgramError, ProgramError};
use thiserror::Error;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use solana_program::msg;

#[derive(Clone, Copy, Debug, Error, FromPrimitive, PartialEq)]
pub enum AmmError {
    #[error("Token X, Y has identical minter")]
    IdenticalMinter,
    #[error("Amm cannot be initialized because it is already being used.")]
    AlreadyInUse,
    #[error("Amount must be not zero")]
    AmountZero,
    #[error("Calculation overflowed the destination number")]
    Overflow,
    #[error("Calculation underflow the destination number")]
    Underflow,
    #[error("Incorrect public key for tokens swap")]
    IncorrectSwapPk,
    #[error("Calculated zero swap amount")]
    CalculatedZeroSwap,
    #[error("Invalid vault")]
    InvalidVault
}

impl From<AmmError> for ProgramError {
    fn from(e: AmmError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for AmmError {
    fn type_of() -> &'static str {
        "AmmError"
    }
}

impl PrintProgramError for AmmError {
    fn print<E>(&self)
        where E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive {
        match self {
            AmmError::IdenticalMinter => msg!("Error: Token X, Y has identical minter"),
            AmmError::AlreadyInUse => msg!("Error: Amm cannot be initialized because it is already being used."),
            AmmError::AmountZero => msg!("Error: Amount must be not zero"),
            AmmError::Overflow => msg!("Error: Calculation overflowed the destination number"),
            AmmError::Underflow => msg!("Error: Calculation underflow the destination number"),
            AmmError::IncorrectSwapPk => msg!("Error: Incorrect public key for tokens swap"),
            AmmError::CalculatedZeroSwap => msg!("Error: Calculated zero swap amount"),
            AmmError::InvalidVault => msg!("Error: Invalid vault"),
        }
    }
}
