use soroban_sdk::{Address, Env};

pub trait LiquidityPoolTrait {
    fn initialize(env: Env, admin: Address, token: Address);

    fn get_total_balance(env: Env) -> i128;

    fn add_lender(env: Env, admin: Address, lender: Address);

    fn remove_lender(env: Env, admin: Address, lender: Address);

    fn add_borrower(env: Env, admin: Address, borrower: Address);

    fn remove_borrower(env: Env, admin: Address, lender: Address);
}
