use soroban_sdk::{Address, Env};

pub trait LiquidityPoolTrait {
    fn initialize(env: Env, admin: Address, token: Address);

    fn balance(env: Env, lender: Address) -> i128;

    fn deposit(env: Env, lender: Address, amount: i128);

    fn withdraw(env: Env, lender: Address, amount: i128);

    fn add_lender(env: Env, admin: Address, lender: Address);

    fn remove_lender(env: Env, admin: Address, lender: Address);

    fn add_borrower(env: Env, admin: Address, borrower: Address);

    fn remove_borrower(env: Env, admin: Address, lender: Address);
}
