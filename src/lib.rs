pub mod error;
pub mod processor;
pub mod instruction;
pub mod state;
pub mod pda;
pub mod swap;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

solana_program::declare_id!("Dybvx3CExEV2zpLSJrcap37Q1cdptvekjr2R3nEu1mTS");