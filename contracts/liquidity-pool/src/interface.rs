use crate::errors::LPError;
use soroban_sdk::{Address, Env};

pub trait LiquidityPoolTrait {
    fn initialize(env: Env, admin: Address, token: Address) -> Result<(), LPError>;

    fn balance(env: Env, lender: Address) -> Result<i128, LPError>;

    fn deposit(env: Env, lender: Address, amount: i128) -> Result<(), LPError>;

    fn withdraw(env: Env, lender: Address, amount: i128) -> Result<(), LPError>;

    fn loan(env: Env, borrower: Address, amount: i128) -> Result<u64, LPError>;

    fn repay_loan(env: Env, borrower: Address, loan_id: u64, amount: i128) -> Result<(), LPError>;

    fn repay_loan_amount(env: Env, borrower: Address, loan_id: u64) -> Result<i128, LPError>;

    fn add_lender(env: Env, lender: Address) -> Result<(), LPError>;

    fn set_lender_status(env: Env, lender: Address, active: bool) -> Result<(), LPError>;

    fn remove_lender(env: Env, lender: Address) -> Result<(), LPError>;

    fn add_borrower(env: Env, borrower: Address) -> Result<(), LPError>;

    fn set_borrower_status(env: Env, borrower: Address, active: bool) -> Result<(), LPError>;

    fn set_borrower_limits(
        env: Env,
        borrower: Address,
        min_amount: i128,
        max_amount: i128,
    ) -> Result<(), LPError>;

    fn remove_borrower(env: Env, lender: Address) -> Result<(), LPError>;
}
