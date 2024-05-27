#![no_std]

mod testutils;
mod types;

use crate::types::DataKey;
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct ClearContract;

#[contractimpl]
impl ClearContract {
    pub fn initialize(env: Env, admin: Address, token: Address) {
        assert!(
            !env.storage().persistent().has(&DataKey::Admin),
            "contract already initialized with an admin"
        );

        env.storage().persistent().set(&DataKey::Admin, &admin);

        env.storage().persistent().set(&DataKey::Token, &token);

        env.storage()
            .persistent()
            .set(&DataKey::TotalBalance, &0i128);
    }

    pub fn get_total_balance(env: &Env) -> i128 {
        env.storage()
            .persistent()
            .get::<_, i128>(&DataKey::TotalBalance)
            .unwrap_or(0)
    }
}

mod test;
