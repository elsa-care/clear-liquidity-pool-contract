#![no_std]

mod event;
mod interface;
mod percentage;
mod storage;
mod testutils;
mod types;

use crate::interface::LiquidityPoolTrait;
use crate::percentage::{calculate_repayment_amount, process_lender_contribution};
use crate::storage::{
    has_admin, has_borrower, has_lender, read_admin, read_contract_balance, read_contributions,
    read_lender, read_loans, read_token, remove_borrower, remove_lender,
    remove_lender_contribution, write_admin, write_borrower, write_contract_balance, write_lender,
    write_lender_contribution, write_loans, write_token,
};
use crate::types::Loan;

use soroban_sdk::{
    contract, contractimpl, contractmeta,
    token::{self},
    Address, Env, Vec,
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

fn generate_id(env: &Env, loans: &Vec<Loan>) -> u64 {
    loop {
        let new_id = env.prng().gen();
        if !loans.iter().any(|loan| loan.id == new_id) {
            return new_id;
        }
    }
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

        event::initialize(&env, admin, token);
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
            contributions.push_back(lender.clone());
            write_lender_contribution(&env, contributions);
        }

        event::deposit(&env, lender, amount);
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

        event::withdraw(&env, lender, amount);
    }

    fn loan(env: Env, borrower: Address, amount: i128) -> u64 {
        borrower.require_auth();

        assert!(amount > 0, "amount must be positive");
        assert!(has_borrower(&env, &borrower), "borrower is not registered");

        let total_balance = read_contract_balance(&env);

        assert!(
            total_balance >= amount,
            "balance not available for the amount requested"
        );

        token_transfer(&env, &env.current_contract_address(), &borrower, &amount);

        let lenders = read_contributions(&env);

        let (lender_contributions, new_lender_amounts) =
            process_lender_contribution(&env, lenders.clone(), &amount, &total_balance);

        let mut loans = read_loans(&env, &borrower);

        let new_loan = Loan {
            id: generate_id(&env, &loans),
            amount,
            start_time: env.ledger().timestamp(),
            contributions: lender_contributions,
        };

        loans.push_back(new_loan.clone());

        for lender in lenders.iter() {
            let new_lender_balance = new_lender_amounts.get(lender.clone()).unwrap();
            write_lender(&env, &lender, &new_lender_balance);
        }

        write_contract_balance(&env, &(total_balance - amount));
        write_loans(&env, &borrower, &loans);
        write_borrower(&env, &borrower, true);

        event::loan(&env, borrower, new_loan.id, amount);

        new_loan.id
    }

    fn repay_loan(env: Env, borrower: Address, loan_id: u64, amount: i128) {
        borrower.require_auth();

        assert!(amount > 0, "amount must be positive");
        assert!(has_borrower(&env, &borrower), "borrower is not registered");

        let mut loans = read_loans(&env, &borrower);
        let (loan_index, mut loan) = loans
            .iter()
            .enumerate()
            .find(|(_, loan)| loan.id == loan_id)
            .map(|(index, loan)| (index, loan.clone()))
            .expect("borrower's loan was not found or exists");

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
            loans.set(loan_index as u32, loan);
        } else {
            loans.remove(loan_index as u32);
        }

        write_loans(&env, &borrower, &loans);
        write_contract_balance(&env, &total_balance);

        event::repay_loan(&env, borrower, loan_id, amount);
    }

    fn repay_loan_amount(env: Env, borrower: Address, loan_id: u64) -> i128 {
        borrower.require_auth();

        assert!(has_borrower(&env, &borrower), "borrower is not registered");

        let loans = read_loans(&env, &borrower);
        let loan = loans
            .iter()
            .find(|loan| loan.id == loan_id)
            .expect("borrower's loan was not found or exists");

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
        event::add_borrower(&env, admin, borrower);
    }

    fn remove_borrower(env: Env, borrower: Address) {
        let admin = read_admin(&env);
        admin.require_auth();

        assert!(has_borrower(&env, &borrower), "borrower is not registered");

        remove_borrower(&env, &borrower);
        event::remove_borrower(&env, admin, borrower);
    }

    fn add_lender(env: Env, lender: Address) {
        let admin = read_admin(&env);
        admin.require_auth();

        assert!(!has_lender(&env, &lender), "lender is already registered");

        write_lender(&env, &lender, &0i128);
        event::add_lender(&env, admin, lender);
    }

    fn remove_lender(env: Env, lender: Address) {
        let admin = read_admin(&env);
        admin.require_auth();

        assert!(has_lender(&env, &lender), "lender is not registered");

        remove_lender(&env, &lender);
        remove_lender_contribution(&env, &lender);
        event::remove_lender(&env, admin, lender);
    }
}

mod test;
