#![no_std]

mod types;

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct ClearContract;

#[contractimpl]
impl ClearContract {
    pub fn initialize(env: Env, admin: Address, token: Address) {
        assert!(
            !env.storage().persistent().has(&types::DataKey::Admin),
            "contract already initialized with an admin"
        );

        env.storage()
            .persistent()
            .set(&types::DataKey::Admin, &admin);

        env.storage()
            .persistent()
            .set(&types::DataKey::Token, &token);

        env.storage()
            .persistent()
            .set(&types::DataKey::TotalBalance, &0i128);
    }

    pub fn get_total_balance(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get::<_, i128>(&types::DataKey::TotalBalance)
            .unwrap_or(0)
    }
}
