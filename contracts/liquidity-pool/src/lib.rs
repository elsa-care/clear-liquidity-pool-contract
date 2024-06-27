#![no_std]

mod interface;
mod percentage;
mod storage;
mod testutils;
mod types;

use crate::interface::LiquidityPoolTrait;
use crate::percentage::process_lender_contribution;
use crate::storage::{
    has_admin, has_borrower, has_lender, has_loan, read_admin, read_contract_balance,
    read_contributions, read_lender, read_loan, read_token, remove_borrower, remove_lender,
    remove_lender_contribution, write_admin, write_borrower, write_contract_balance, write_lender,
    write_lender_contribution, write_loan, write_token,
};
use crate::types::Loan;

use soroban_sdk::{
    contract, contractimpl,
    token::{self},
    Address, Env,
};

fn token_transfer(env: &Env, from: &Address, to: &Address, amount: &i128) {
    let token_id = read_token(env);
    let token = token::Client::new(env, &token_id);
    token.transfer(from, to, amount);
}

fn calculate_fees(env: &Env, loan: &Loan) -> i128 {
    let now_ledger = env.ledger().timestamp();
    let start_time = loan.start_time;
    let interest_rate_per_day = 1;
    let seconds_per_day = 86400;

    let duration_days = (now_ledger - start_time) / seconds_per_day;

    loan.amount * (interest_rate_per_day * duration_days) as i128 / 100
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

        let mut total_balance = read_contract_balance(&env);
        let mut lender_balance = read_lender(&env, &lender);

        total_balance += amount;
        lender_balance += amount;

        write_contract_balance(&env, &total_balance);
        write_lender(&env, &lender, &lender_balance);

        let mut lender_contribution = read_contributions(&env);

        lender_contribution = process_lender_contribution(
            0,
            &lender,
            lender_contribution,
            &lender_balance,
            &total_balance,
            &(total_balance - amount),
        );

        write_lender_contribution(&env, lender_contribution);
    }

    fn withdraw(env: Env, lender: Address, amount: i128) {
        lender.require_auth();

        assert!(has_lender(&env, &lender), "lender is not registered");

        let mut total_balance = read_contract_balance(&env);
        let mut lender_balance = read_lender(&env, &lender);

        assert!(amount > 0, "amount must be positive");
        assert!(
            lender_balance >= amount,
            "balance not available for the amount requested"
        );
        assert!(total_balance >= amount, "balance currently unavailable");

        token_transfer(&env, &env.current_contract_address(), &lender, &amount);

        total_balance -= amount;
        lender_balance -= amount;

        write_contract_balance(&env, &total_balance);
        write_lender(&env, &lender, &lender_balance);

        let mut lender_contribution = read_contributions(&env);

        if lender_balance > 0 {
            lender_contribution = process_lender_contribution(
                1,
                &lender,
                lender_contribution,
                &lender_balance,
                &total_balance,
                &(total_balance + amount),
            );
        } else {
            lender_contribution.remove(lender.clone());
        }

        write_lender_contribution(&env, lender_contribution);
    }

    fn loan(env: Env, borrower: Address, amount: i128) {
        borrower.require_auth();

        assert!(amount > 0, "amount must be positive");
        assert!(has_borrower(&env, &borrower), "borrower is not registered");
        assert!(
            !has_loan(&env, &borrower),
            "borrower already has an active loan"
        );

        let total_balance = read_contract_balance(&env);

        assert!(
            total_balance >= amount,
            "balance not available for the amount requested"
        );

        token_transfer(&env, &env.current_contract_address(), &borrower, &amount);

        let lenders = read_contributions(&env);

        let new_loan = Loan {
            amount,
            start_time: env.ledger().timestamp(),
            contributions: lenders,
        };

        write_contract_balance(&env, &(total_balance - amount));
        write_loan(&env, &borrower, &new_loan);
        write_borrower(&env, &borrower, true);
    }

    fn repay_loan_amount(env: Env, borrower: Address) -> i128 {
        borrower.require_auth();

        assert!(has_borrower(&env, &borrower), "borrower is not registered");

        let loan = match read_loan(&env, &borrower) {
            Some(loan) => loan,
            None => panic!("borrower has no active loan"),
        };

        loan.amount + calculate_fees(&env, &loan)
    }

    fn add_borrower(env: Env, admin: Address, borrower: Address) {
        assert_eq!(
            read_admin(&env),
            admin,
            "only the stored admin can add borrowers"
        );

        assert!(
            !has_borrower(&env, &borrower),
            "borrower is already registered"
        );

        write_borrower(&env, &borrower, false);
    }

    fn remove_borrower(env: Env, admin: Address, borrower: Address) {
        assert_eq!(
            read_admin(&env),
            admin,
            "only the stored admin can add borrowers"
        );

        assert!(has_borrower(&env, &borrower), "borrower is not registered");

        remove_borrower(&env, &borrower);
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
        remove_lender_contribution(&env, &lender);
    }
}

mod test;
