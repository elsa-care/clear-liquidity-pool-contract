#![no_std]

mod interface;
mod percentage;
mod storage;
mod testutils;
mod types;

use crate::interface::LiquidityPoolTrait;
use crate::percentage::{calculate_repayment_amount, process_lender_contribution};
use crate::storage::{
    has_admin, has_borrower, has_lender, has_loan, read_admin, read_contract_balance,
    read_contributions, read_lender, read_loan, read_token, remove_borrower, remove_lender,
    remove_lender_contribution, remove_loan, write_admin, write_borrower, write_contract_balance,
    write_lender, write_lender_contribution, write_loan, write_token,
};
use crate::types::Loan;

use soroban_sdk::{
    contract, contractimpl, contractmeta,
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
    let interest_rate_per_day = 10;
    let seconds_per_day = 86400;

    let duration_days = (now_ledger - start_time) / seconds_per_day;

    loan.amount * (interest_rate_per_day * duration_days) as i128 / 100_000
}

contractmeta!(
    key = "Description",
    val = "Liquidity pool for loans with a daily fee of 0.1%"
);

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

        let mut contributions = read_contributions(&env);

        if !contributions.contains(lender.clone()) {
            contributions.push_back(lender);
            write_lender_contribution(&env, contributions);
        }
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

        if lender_balance <= 0 {
            remove_lender_contribution(&env, &lender);
        }
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

        let (lender_contributions, new_lender_amounts) =
            process_lender_contribution(&env, lenders.clone(), &amount, &total_balance);

        let new_loan = Loan {
            amount,
            start_time: env.ledger().timestamp(),
            contributions: lender_contributions,
        };

        for lender in lenders.iter() {
            let new_lender_balance = new_lender_amounts.get(lender.clone()).unwrap();
            write_lender(&env, &lender, &new_lender_balance);
        }

        write_contract_balance(&env, &(total_balance - amount));
        write_loan(&env, &borrower, &new_loan);
        write_borrower(&env, &borrower, true);
    }

    fn repay_loan(env: Env, borrower: Address, amount: i128) {
        borrower.require_auth();

        assert!(amount > 0, "amount must be positive");
        assert!(has_borrower(&env, &borrower), "borrower is not registered");

        let mut loan = match read_loan(&env, &borrower) {
            Some(loan) => loan,
            None => panic!("borrower has no active loan"),
        };

        token_transfer(&env, &borrower, &env.current_contract_address(), &amount);

        for (lender, percentage) in loan.contributions.iter() {
            let lender_balance = read_lender(&env, &lender);
            let repay_lender_amount =
                lender_balance + calculate_repayment_amount(amount, percentage);
            write_lender(&env, &lender, &repay_lender_amount);
        }

        let repay_loan_amount = loan.amount + calculate_fees(&env, &loan);
        let mut total_balance = read_contract_balance(&env);
        total_balance += amount;

        if (repay_loan_amount - amount) > 0 {
            loan.amount = repay_loan_amount - amount;
            write_loan(&env, &borrower, &loan);
        } else {
            write_borrower(&env, &borrower, false);
            remove_loan(&env, &borrower);
        }

        write_contract_balance(&env, &total_balance);
    }

    fn repay_loan_amount(env: Env, borrower: Address) -> i128 {
        assert!(has_borrower(&env, &borrower), "borrower is not registered");

        let loan = match read_loan(&env, &borrower) {
            Some(loan) => loan,
            None => panic!("borrower has no active loan"),
        };

        loan.amount + calculate_fees(&env, &loan)
    }

    fn add_borrower(env: Env, borrower: Address) {
        let admin = read_admin(&env);
        admin.require_auth();

        assert!(
            !has_borrower(&env, &borrower),
            "borrower is already registered"
        );

        write_borrower(&env, &borrower, false);
    }

    fn remove_borrower(env: Env, borrower: Address) {
        let admin = read_admin(&env);
        admin.require_auth();

        assert!(has_borrower(&env, &borrower), "borrower is not registered");

        remove_borrower(&env, &borrower);
    }

    fn add_lender(env: Env, lender: Address) {
        let admin = read_admin(&env);
        admin.require_auth();

        assert!(!has_lender(&env, &lender), "lender is already registered");

        write_lender(&env, &lender, &0i128);
    }

    fn remove_lender(env: Env, lender: Address) {
        let admin = read_admin(&env);
        admin.require_auth();

        assert!(has_lender(&env, &lender), "lender is not registered");

        remove_lender(&env, &lender);
        remove_lender_contribution(&env, &lender);
    }
}

mod test;
