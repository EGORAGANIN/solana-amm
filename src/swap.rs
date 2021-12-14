use num_traits::ToPrimitive;
use solana_program::pubkey::Pubkey;
use spl_math::checked_ceil_div::CheckedCeilDiv;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwapResult {
    pub take_amount: u64,
    pub return_amount: u64,
}

pub enum SwapDirection {
    XtoY,
    YtoX,
}

impl SwapDirection {

    pub fn new(swap_pk: &Pubkey, x_pk: &Pubkey, y_pk: &Pubkey) -> Option<SwapDirection> {
        if swap_pk == x_pk {
            Some(SwapDirection::XtoY)
        } else if swap_pk == y_pk {
            Some(SwapDirection::YtoX)
        } else {
            None
        }
    }
}

pub fn calc_swap(
    add_source_amount: u64,
    source_amount: u64,
    destination_amount: u64,
) -> Option<SwapResult> {
    let add_source_amount = add_source_amount.to_u128()?;
    let source_amount = source_amount.to_u128()?;
    let destination_amount = destination_amount.to_u128()?;

    // K = X * Y
    let invariant = source_amount.checked_mul(destination_amount)?;

    // (X + dX)
    let new_source_amount = source_amount.checked_add(add_source_amount)?;

    // ((Y - dY), M(updated) = K / M
    let (new_destination_amount, new_source_amount) = invariant.checked_ceil_div(new_source_amount)?;

    //  dX = (X + dX) - X
    let take_amount_x = new_source_amount.checked_sub(source_amount)?.to_u64()?;
    if take_amount_x == 0 {
        return None
    }

    //  dY = Y - (Y - dY)
    let return_amount_y = destination_amount.checked_sub(new_destination_amount)?.to_u64()?;
    if return_amount_y == 0 {
        return None
    }

    Some(SwapResult { take_amount: take_amount_x, return_amount: return_amount_y })
}
