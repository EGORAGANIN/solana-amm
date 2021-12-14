use solana_program::account_info::AccountInfo;
use solana_program::entrypoint;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::PrintProgramError;
use solana_program::pubkey::Pubkey;
use crate::error::AmmError;
use crate::processor::Processor;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {
    if let Err(e) = Processor::process(program_id, accounts, instruction_data) {
        e.print::<AmmError>();
        return Err(e);
    }
    Ok(())
}