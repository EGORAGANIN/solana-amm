use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::{msg, system_instruction};
use solana_program::pubkey::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use crate::error::AmmError;
use crate::state::Vault;
use crate::instruction::AmmInstruction;
use crate::id;
use crate::pda::{VAULT_SEED, SPL_TOKEN_X_OWNER_SEED, SPL_TOKEN_Y_OWNER_SEED, Pda};
use crate::swap::{calc_swap, SwapDirection};

pub struct Processor;

impl Processor {
    pub fn process(_program_id: &Pubkey,
                   accounts: &[AccountInfo],
                   instruction_data: &[u8]) -> ProgramResult {
        let ix = AmmInstruction::try_from_slice(instruction_data)?;
        match ix {
            AmmInstruction::InitMarket { amount_x, amount_y } => {
                msg!("AmmInstruction: InitMarket");
                Self::process_init_market(amount_x, amount_y, accounts)
            }
            AmmInstruction::Swap { amount, minter_pk } => {
                msg!("AmmInstruction: Swap");
                Self::process_swap(amount, minter_pk, accounts)
            }
        }
    }

    fn process_init_market(
        amount_x: u64,
        amount_y: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        msg!("process_init_market: Reading accounts");
        let acc_iter = &mut accounts.iter();

        // user accounts
        let user_owner_token_x_info = next_account_info(acc_iter)?;
        let user_owner_token_y_info = next_account_info(acc_iter)?;
        let user_payer_info = next_account_info(acc_iter)?;
        let user_token_x_info = next_account_info(acc_iter)?;
        let user_token_y_info = next_account_info(acc_iter)?;
        let minter_x_info = next_account_info(acc_iter)?;
        let minter_y_info = next_account_info(acc_iter)?;

        // contract accounts
        let pda_token_x_info = next_account_info(acc_iter)?;
        let pda_token_y_info = next_account_info(acc_iter)?;
        let pda_owner_token_x_info = next_account_info(acc_iter)?;
        let pda_owner_token_y_info = next_account_info(acc_iter)?;
        let pda_vault_info = next_account_info(acc_iter)?;

        // service accounts
        let rent_info = next_account_info(acc_iter)?;
        let rent = Rent::from_account_info(rent_info)?;
        let system_info = next_account_info(acc_iter)?;
        let spl_token_program_info = next_account_info(acc_iter)?;
        let spl_associated_token_program_info = next_account_info(acc_iter)?;


        msg!("process_init_market: Verifying accounts");
        if !user_owner_token_x_info.is_signer {
            msg!("Error: Required signature for user SPL token X owner");
            return Err(ProgramError::MissingRequiredSignature);
        }
        if !user_owner_token_y_info.is_signer {
            msg!("Error: Required signature for user SPL token Y owner");
            return Err(ProgramError::MissingRequiredSignature);
        }
        if !user_payer_info.is_signer {
            msg!("Error: Required signature for user payer");
            return Err(ProgramError::MissingRequiredSignature);
        }
        if minter_x_info.key == minter_y_info.key {
            return Err(AmmError::IdenticalMinter.into());
        }

        let pda = Pda::generate(minter_x_info.key, minter_y_info.key);
        let pda_owner_token_x_pk = pda.pda_owner_token_x.0;
        let pda_owner_token_y_pk = pda.pda_owner_token_y.0;
        let pda_associated_token_x_pk = pda.pda_token_x_pk;
        let pda_associated_token_y_pk = pda.pda_token_y_pk;
        let (vault_pk, vault_bump) = pda.vault;

        if *pda_owner_token_x_info.key != pda_owner_token_x_pk {
            msg!("Error: Pda owner token X address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }
        if *pda_owner_token_y_info.key != pda_owner_token_y_pk {
            msg!("Error: Pda owner token Y address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }
        if *pda_token_x_info.key != pda_associated_token_x_pk {
            msg!("Error: Pda token X address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }
        if *pda_token_y_info.key != pda_associated_token_y_pk {
            msg!("Error: Pda token Y address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }
        if *pda_vault_info.key != vault_pk {
            msg!("Error: Pda vault address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }

        if amount_x == 0 || amount_y == 0 {
            return Err(AmmError::AmountZero.into());
        }


        if pda_token_x_info.data_is_empty() {
            msg!("process_init_market: Creating pda token X associated account");
            let create_associated_token_x_acc_ix = spl_associated_token_account::create_associated_token_account(
                user_payer_info.key,
                pda_owner_token_x_info.key,
                minter_x_info.key,
            );
            invoke(
                &create_associated_token_x_acc_ix,
                &[
                    user_payer_info.clone(),
                    pda_token_x_info.clone(),
                    pda_owner_token_x_info.clone(),
                    minter_x_info.clone(),
                    system_info.clone(),
                    spl_token_program_info.clone(),
                    rent_info.clone(),
                    spl_associated_token_program_info.clone()
                ],
            )?;
        } else {
            return Err(AmmError::AlreadyInUse.into());
        }

        if pda_token_y_info.data_is_empty() {
            msg!("process_init_market: Creating pda token Y associated account");
            let create_associated_token_y_acc_ix = spl_associated_token_account::create_associated_token_account(
                user_payer_info.key,
                pda_owner_token_y_info.key,
                minter_y_info.key,
            );
            invoke(
                &create_associated_token_y_acc_ix,
                &[
                    user_payer_info.clone(),
                    pda_token_y_info.clone(),
                    pda_owner_token_y_info.clone(),
                    minter_y_info.clone(),
                    system_info.clone(),
                    spl_token_program_info.clone(),
                    rent_info.clone(),
                    spl_associated_token_program_info.clone()
                ],
            )?;
        } else {
            return Err(AmmError::AlreadyInUse.into());
        }

        msg!("process_init_market: Transfer amount_x={} to pda token X associated account", amount_x);
        let transfer_token_x_ix = spl_token::instruction::transfer(
            spl_token_program_info.key,
            user_token_x_info.key,
            pda_token_x_info.key,
            user_owner_token_x_info.key,
            &[&user_owner_token_x_info.key],
            amount_x,
        )?;
        invoke(
            &transfer_token_x_ix,
            &[
                spl_token_program_info.clone(),
                user_token_x_info.clone(),
                pda_token_x_info.clone(),
                user_owner_token_x_info.clone()
            ],
        )?;

        msg!("process_init_market: Transfer amount_y={} to pda token Y associated account", amount_y);
        let transfer_token_y_ix = spl_token::instruction::transfer(
            spl_token_program_info.key,
            user_token_y_info.key,
            pda_token_y_info.key,
            user_owner_token_y_info.key,
            &[&user_owner_token_y_info.key],
            amount_y,
        )?;
        invoke(
            &transfer_token_y_ix,
            &[
                spl_token_program_info.clone(),
                user_token_y_info.clone(),
                pda_token_y_info.clone(),
                user_owner_token_y_info.clone()
            ],
        )?;


        if pda_vault_info.data_is_empty() {
            msg!("process_init_market: Creating vault account");
            let vault = Vault { token_x_amount: 0, token_y_amount: 0 };
            let space = vault.try_to_vec()?.len();
            let rent_value = rent.minimum_balance(space);
            let create_vault_acc_ix = system_instruction::create_account(
                user_payer_info.key,
                pda_vault_info.key,
                rent_value,
                space as u64,
                &id(),
            );
            invoke_signed(
                &create_vault_acc_ix,
                &[user_payer_info.clone(), pda_vault_info.clone(), system_info.clone()],
                &[&[
                    VAULT_SEED,
                    &minter_x_info.key.to_bytes(),
                    &minter_y_info.key.to_bytes(),
                    &spl_token::id().to_bytes(),
                    &[vault_bump]
                ]],
            )?;
        } else {
            return Err(AmmError::AlreadyInUse.into());
        }


        let mut vault: Vault = Vault::try_from_slice(&pda_vault_info.data.borrow())?;
        msg!(
            "process_init_market: Current amount_x={}, amount_y={} from vault account",
            vault.token_x_amount, vault.token_y_amount
        );
        vault.token_x_amount = amount_x;
        vault.token_y_amount = amount_y;

        vault.serialize(&mut &mut pda_vault_info.data.borrow_mut()[..])?;
        msg!(
            "process_init_market: Saved new amount_x={}, amount_y={} to vault account",
            vault.token_x_amount, vault.token_y_amount
        );

        Ok(())
    }

    fn process_swap(
        amount: u64,
        minter_pk: Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        msg!("process_swap: Reading accounts");
        let acc_iter = &mut accounts.iter();

        // user accounts
        let user_owner_token_info = next_account_info(acc_iter)?;
        let user_token_x_info = next_account_info(acc_iter)?;
        let user_token_y_info = next_account_info(acc_iter)?;
        let minter_x_info = next_account_info(acc_iter)?;
        let minter_y_info = next_account_info(acc_iter)?;

        // contract accounts
        let pda_token_x_info = next_account_info(acc_iter)?;
        let pda_token_y_info = next_account_info(acc_iter)?;
        let pda_owner_token_x_info = next_account_info(acc_iter)?;
        let pda_owner_token_y_info = next_account_info(acc_iter)?;
        let pda_vault_info = next_account_info(acc_iter)?;

        // service accounts
        let spl_token_program_info = next_account_info(acc_iter)?;

        msg!("process_swap: Verifying accounts");
        if !user_owner_token_info.is_signer {
            msg!("Error: Required signature for user SPL token owner");
            return Err(ProgramError::MissingRequiredSignature);
        }
        if minter_x_info.key == minter_y_info.key {
            return Err(AmmError::IdenticalMinter.into());
        }
        if minter_pk != *minter_x_info.key && minter_pk != *minter_y_info.key {
            return Err(AmmError::IncorrectSwapPk.into());
        }

        let pda = Pda::generate(minter_x_info.key, minter_y_info.key);
        let (pda_owner_token_x_pk, pda_owner_token_x_bump) = pda.pda_owner_token_x;
        let (pda_owner_token_y_pk, pda_owner_token_y_bump) = pda.pda_owner_token_y;
        let pda_associated_token_x_pk = pda.pda_token_x_pk;
        let pda_associated_token_y_pk = pda.pda_token_y_pk;
        let vault_pk = pda.vault.0;

        if *pda_owner_token_x_info.key != pda_owner_token_x_pk {
            msg!("Error: Pda owner token X address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }
        if *pda_owner_token_y_info.key != pda_owner_token_y_pk {
            msg!("Error: Pda owner token Y address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }
        if *pda_token_x_info.key != pda_associated_token_x_pk {
            msg!("Error: Pda token X address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }
        if *pda_token_y_info.key != pda_associated_token_y_pk {
            msg!("Error: Pda token Y address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }
        if *pda_vault_info.key != vault_pk {
            msg!("Error: Pda vault address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }

        if amount == 0 {
            return Err(AmmError::AmountZero.into());
        }

        let mut vault: Vault = Vault::try_from_slice(&pda_vault_info.data.borrow())
            .map_err(|_| Into::<ProgramError>::into(AmmError::InvalidVault))?;
        msg!(
            "process_swap: Current amount_x={}, amount_y={} from vault account",
            vault.token_x_amount, vault.token_y_amount
        );

        let swap_direction = SwapDirection::new(&minter_pk, minter_x_info.key, minter_y_info.key)
            .ok_or(AmmError::IncorrectSwapPk)?;

        let swap_result = match swap_direction {
            SwapDirection::XtoY => calc_swap(
                amount,
                vault.token_x_amount,
                vault.token_y_amount,
            ),
            SwapDirection::YtoX => calc_swap(
                amount,
                vault.token_y_amount,
                vault.token_x_amount,
            )
        }.ok_or(AmmError::CalculatedZeroSwap)?;

        match swap_direction {
            SwapDirection::XtoY => {
                Self::transfer_to_market(
                    spl_token_program_info,
                    user_token_x_info,
                    pda_token_x_info,
                    user_owner_token_info,
                    swap_result.take_amount,
                )?;
                Self::transfer_to_user(
                    spl_token_program_info,
                    pda_token_y_info,
                    user_token_y_info,
                    pda_owner_token_y_info,
                    swap_result.return_amount,
                    &[&[
                        SPL_TOKEN_Y_OWNER_SEED,
                        &minter_x_info.key.to_bytes(),
                        &minter_y_info.key.to_bytes(),
                        &spl_token::id().to_bytes(),
                        &[pda_owner_token_y_bump]
                    ]],
                )?;
            }
            SwapDirection::YtoX => {
                Self::transfer_to_market(
                    spl_token_program_info,
                    user_token_y_info,
                    pda_token_y_info,
                    user_owner_token_info,
                    swap_result.take_amount,
                )?;
                Self::transfer_to_user(
                    spl_token_program_info,
                    pda_token_x_info,
                    user_token_x_info,
                    pda_owner_token_x_info,
                    swap_result.return_amount,
                    &[&[
                        SPL_TOKEN_X_OWNER_SEED,
                        &minter_x_info.key.to_bytes(),
                        &minter_y_info.key.to_bytes(),
                        &spl_token::id().to_bytes(),
                        &[pda_owner_token_x_bump]
                    ]],
                )?;
            }
        }

        let (nex_token_x_amount, nex_token_y_amount) = match swap_direction {
            SwapDirection::XtoY => (
                vault.token_x_amount.checked_add(swap_result.take_amount)
                    .ok_or(AmmError::Overflow)?,
                vault.token_y_amount.checked_sub(swap_result.return_amount)
                    .ok_or(AmmError::Underflow)?
            ),
            SwapDirection::YtoX => (
                vault.token_y_amount.checked_add(swap_result.take_amount)
                    .ok_or(AmmError::Overflow)?,
                vault.token_x_amount.checked_sub(swap_result.return_amount)
                    .ok_or(AmmError::Underflow)?
            )
        };

        vault.token_x_amount = nex_token_x_amount;
        vault.token_y_amount = nex_token_y_amount;

        vault.serialize(&mut &mut pda_vault_info.data.borrow_mut()[..])?;
        msg!(
            "process_swap: Saved new amount_x={}, amount_y={} to vault account",
            vault.token_x_amount, vault.token_y_amount
        );

        Ok(())
    }

    fn transfer_to_market<'a>(
        spl_token_program_info: &AccountInfo<'a>,
        source_info: &AccountInfo<'a>,
        destination_info: &AccountInfo<'a>,
        authority_info: &AccountInfo<'a>,
        amount: u64,
    ) -> ProgramResult {
        msg!("process_swap: Transfer amount={} to pda token associated account", amount);
        let transfer_token_ix = spl_token::instruction::transfer(
            spl_token_program_info.key,
            source_info.key,
            destination_info.key,
            authority_info.key,
            &[&authority_info.key],
            amount,
        )?;
        invoke(
            &transfer_token_ix,
            &[
                spl_token_program_info.clone(),
                source_info.clone(),
                destination_info.clone(),
                authority_info.clone()
            ],
        )
    }

    fn transfer_to_user<'a>(
        spl_token_program_info: &AccountInfo<'a>,
        source_info: &AccountInfo<'a>,
        destination_info: &AccountInfo<'a>,
        authority_info: &AccountInfo<'a>,
        amount: u64,
        signers_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        msg!("process_init_market: Transfer amount={} to user token account", amount);
        let transfer_token_ix = spl_token::instruction::transfer(
            spl_token_program_info.key,
            source_info.key,
            destination_info.key,
            authority_info.key,
            &[&authority_info.key],
            amount,
        )?;
        invoke_signed(
            &transfer_token_ix,
            &[
                spl_token_program_info.clone(),
                source_info.clone(),
                destination_info.clone(),
                authority_info.clone()
            ],
            signers_seeds,
        )
    }
}
