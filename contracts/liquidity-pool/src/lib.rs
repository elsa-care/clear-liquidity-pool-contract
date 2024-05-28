#![no_std]

mod interface;
mod storage;
mod testutils;
mod types;

use crate::{
    interface::LiquidityPoolTrait,
    storage::{get_admin, get_all_borrowers},
    types::DataKey,
};
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct LiquidityPoolContract;

#[contractimpl]
impl LiquidityPoolTrait for LiquidityPoolContract {
    fn initialize(env: Env, admin: Address, token: Address) {
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

    fn get_total_balance(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get::<_, i128>(&DataKey::TotalBalance)
            .unwrap_or(0)
    }

    fn add_borrower(env: Env, admin: Address, borrower: Address) {
        assert_eq!(
            get_admin(&env),
            admin,
            "only the stored admin can add borrowers"
        );

        let mut borrowers = get_all_borrowers(&env);

        borrowers.push_back(borrower);

        env.storage()
            .persistent()
            .set(&DataKey::Borrowers, &borrowers);
    }

    fn remove_borrower(env: Env, admin: Address, borrower: Address) {
        assert_eq!(
            get_admin(&env),
            admin,
            "only the stored admin can add borrowers"
        );

        let mut borrowers = get_all_borrowers(&env);

        if let Some(index) = borrowers.iter().position(|address| address == borrower) {
            borrowers.remove(index as u32);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Borrowers, &borrowers);
    }
}

mod test;
