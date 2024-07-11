use soroban_sdk::{Address, Env};

pub trait LiquidityPoolTrait {
    fn initialize(env: Env, admin: Address, token: Address);

    fn balance(env: Env, lender: Address) -> i128;

    fn deposit(env: Env, lender: Address, amount: i128);

    fn withdraw(env: Env, lender: Address, amount: i128);

    fn loan(env: Env, borrower: Address, amount: i128);

    fn repay_loan(env: Env, borrower: Address, amount: i128);

    fn repay_loan_amount(env: Env, borrower: Address) -> i128;

    fn add_lender(env: Env, lender: Address);

    fn remove_lender(env: Env, lender: Address);

    fn add_borrower(env: Env, borrower: Address);

    fn remove_borrower(env: Env, lender: Address);
}
