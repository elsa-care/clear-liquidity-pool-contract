#![no_std]

mod interface;
mod storage;
mod testutils;
mod types;

use crate::interface::LiquidityPoolTrait;
use crate::storage::{
    get_all_borrowers, has_admin, has_lender, read_admin, read_contract_balance, remove_lender,
    write_admin, write_contract_balance, write_lender, write_token,
};
use crate::types::DataKey;

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct LiquidityPoolContract;

#[contractimpl]
impl LiquidityPoolTrait for LiquidityPoolContract {
    fn initialize(env: Env, admin: Address, token: Address) {
        assert!(
            !has_admin(&env),
            "contract already initialized with an admin"
        );

        write_admin(&env, &admin);
        write_token(&env, &token);
        write_contract_balance(&env, &0i128);
    }

    fn get_total_balance(env: Env) -> i128 {
        read_contract_balance(&env)
    }

    fn add_borrower(env: Env, admin: Address, borrower: Address) {
        assert_eq!(
            read_admin(&env),
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
            read_admin(&env),
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

    fn add_lender(env: Env, admin: Address, lender: Address) {
        assert_eq!(
            read_admin(&env),
            admin,
            "only the stored admin can add lenders"
        );
        assert!(!has_lender(&env, &lender), "lender is already registered");

        write_lender(&env, &lender, &0i128);
    }

    fn remove_lender(env: Env, admin: Address, lender: Address) {
        assert_eq!(
            read_admin(&env),
            admin,
            "only the stored admin can add lenders"
        );
        assert!(has_lender(&env, &lender), "lender is not registered");

        remove_lender(&env, &lender);
    }
}

mod test;
