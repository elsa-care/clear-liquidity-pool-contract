use crate::errors::LPError;
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};

pub fn addition<T>(number: &T, addend: &T) -> Result<T, LPError>
where
    T: Copy + CheckedAdd,
{
    number.checked_add(addend).ok_or(LPError::OverflowError)
}

pub fn subtraction<T>(number: &T, subtrahend: &T) -> Result<T, LPError>
where
    T: Copy + CheckedSub,
{
    number
        .checked_sub(subtrahend)
        .ok_or(LPError::UnderflowError)
}

pub fn multiply<T>(number: &T, multiplier: &T) -> Result<T, LPError>
where
    T: Copy + CheckedMul,
{
    number.checked_mul(multiplier).ok_or(LPError::OverflowError)
}

pub fn division<T>(number: &T, dividend: &T) -> Result<T, LPError>
where
    T: Copy + CheckedDiv,
{
    number.checked_div(dividend).ok_or(LPError::UnderflowError)
}
