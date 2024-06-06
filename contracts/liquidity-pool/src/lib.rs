#![no_std]

mod interface;
mod storage;
mod testutils;
mod types;

use crate::interface::LiquidityPoolTrait;
use crate::storage::{
    get_all_borrowers, has_admin, has_lender, read_admin, read_contract_balance, read_lender,
    read_token, remove_lender, write_admin, write_contract_balance, write_lender, write_token,
};
use crate::types::DataKey;

use soroban_sdk::{
    contract, contractimpl,
    token::{self},
    Address, Env,
};

fn token_transfer(env: &Env, from: &Address, to: &Address, amount: &i128) {
    let token_id = read_token(&env);
    let token = token::Client::new(&env, &token_id);
    token.transfer(&from, &to, &amount);
}

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

    fn balance(env: Env, address: Address) -> i128 {
        if address == read_admin(&env) {
            return read_contract_balance(&env);
        };

        if has_lender(&env, &address) {
            return read_lender(&env, &address);
        }

        panic!("address is not registered");
    }

    fn deposit(env: Env, lender: Address, amount: i128) {
        lender.require_auth();

        assert!(has_lender(&env, &lender), "lender is not registered");
        assert!(amount > 0, "amount must be positive");

        token_transfer(&env, &lender, &env.current_contract_address(), &amount);

        let total_balance = read_contract_balance(&env);
        let lender_balance = read_lender(&env, &lender);

        write_contract_balance(&env, &(total_balance + amount));
        write_lender(&env, &lender, &(lender_balance + amount));
    }

    fn withdraw(env: Env, lender: Address, amount: i128) {
        lender.require_auth();

        assert!(has_lender(&env, &lender), "lender is not registered");

        let total_balance = read_contract_balance(&env);
        let lender_balance = read_lender(&env, &lender);

        assert!(amount > 0, "amount must be positive");
        assert!(
            lender_balance >= amount,
            "balance not available for the amount requested"
        );
        assert!(total_balance >= amount, "balance currently unavailable");

        token_transfer(&env, &env.current_contract_address(), &lender, &amount);

        write_contract_balance(&env, &(total_balance - amount));
        write_lender(&env, &lender, &(lender_balance - amount));
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
