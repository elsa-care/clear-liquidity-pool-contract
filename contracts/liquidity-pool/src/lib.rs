#![no_std]

mod errors;
mod event;
mod interface;
mod percentage;
mod storage;
mod testutils;
mod types;

use crate::errors::LPError;
use crate::interface::LiquidityPoolTrait;
use crate::percentage::{calculate_repayment_amount, process_lender_contribution};
use crate::storage::{
    check_admin, has_admin, has_borrower, has_lender, read_admin, read_contract_balance,
    read_contributions, read_lender, read_loans, read_token, remove_borrower, remove_lender,
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

fn check_nonnegative_amount(amount: i128) -> Result<(), LPError> {
    if amount < 0 {
        return Err(LPError::AmountMustBePositive);
    }

    Ok(())
}

contractmeta!(
    key = "Description",
    val = "Liquidity pool for loans with a daily fee of 0.1%"
);

#[contract]
pub struct LiquidityPoolContract;

#[contractimpl]
impl LiquidityPoolTrait for LiquidityPoolContract {
    fn initialize(env: Env, admin: Address, token: Address) -> Result<(), LPError> {
        if has_admin(&env) {
            return Err(LPError::AlreadyInitialized);
        }

        write_admin(&env, &admin);
        write_token(&env, &token);
        write_contract_balance(&env, &0i128);

        event::initialize(&env, admin, token);
        Ok(())
    }

    fn balance(env: Env, address: Address) -> Result<i128, LPError> {
        if address == read_admin(&env)? {
            return Ok(read_contract_balance(&env));
        };

        if has_lender(&env, &address) {
            return Ok(read_lender(&env, &address));
        }

        Err(LPError::AddressNotRegistered)
    }

    fn deposit(env: Env, lender: Address, amount: i128) -> Result<(), LPError> {
        lender.require_auth();

        check_nonnegative_amount(amount)?;
        if !has_lender(&env, &lender) {
            return Err(LPError::LenderNotRegistered);
        }

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
        Ok(())
    }

    fn withdraw(env: Env, lender: Address, amount: i128) -> Result<(), LPError> {
        lender.require_auth();

        if !has_lender(&env, &lender) {
            return Err(LPError::LenderNotRegistered);
        }

        let mut total_balance = read_contract_balance(&env);
        let mut lender_balance = read_lender(&env, &lender);

        check_nonnegative_amount(amount)?;

        if amount > lender_balance {
            return Err(LPError::InsufficientBalance);
        }

        if amount > total_balance {
            return Err(LPError::BalanceNotAvailableForAmountRequested);
        }

        token_transfer(&env, &env.current_contract_address(), &lender, &amount);

        total_balance -= amount;
        lender_balance -= amount;

        write_contract_balance(&env, &total_balance);
        write_lender(&env, &lender, &lender_balance);

        if lender_balance <= 0 {
            remove_lender_contribution(&env, &lender);
        }

        event::withdraw(&env, lender, amount);
        Ok(())
    }

    fn loan(env: Env, borrower: Address, amount: i128) -> Result<u64, LPError> {
        borrower.require_auth();

        check_nonnegative_amount(amount)?;
        if !has_borrower(&env, &borrower) {
            return Err(LPError::BorrowerNotRegistered);
        }

        let total_balance = read_contract_balance(&env);

        if amount > total_balance {
            return Err(LPError::BalanceNotAvailableForAmountRequested);
        }

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
        Ok(new_loan.id)
    }

    fn repay_loan(env: Env, borrower: Address, loan_id: u64, amount: i128) -> Result<(), LPError> {
        borrower.require_auth();

        check_nonnegative_amount(amount)?;

        if !has_borrower(&env, &borrower) {
            return Err(LPError::BorrowerNotRegistered);
        }

        let mut loans = read_loans(&env, &borrower);
        let (loan_index, mut loan) = loans
            .iter()
            .enumerate()
            .find(|(_, loan)| loan.id == loan_id)
            .map(|(index, loan)| (index, loan.clone()))
            .ok_or(LPError::LoanNotFoundOrExists)?;

        let admin = read_admin(&env)?;
        let total_fees = calculate_fees(&env, &loan);
        let admin_fees = total_fees / 10;
        let amount_for_lenders = amount - admin_fees;

        token_transfer(&env, &borrower, &env.current_contract_address(), &amount);
        token_transfer(&env, &env.current_contract_address(), &admin, &admin_fees);

        for (lender, percentage) in loan.contributions.iter() {
            let lender_balance = read_lender(&env, &lender);
            let repay_lender_amount =
                lender_balance + calculate_repayment_amount(amount_for_lenders, percentage);
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
        Ok(())
    }

    fn repay_loan_amount(env: Env, borrower: Address, loan_id: u64) -> Result<i128, LPError> {
        borrower.require_auth();

        if !has_borrower(&env, &borrower) {
            return Err(LPError::BorrowerNotRegistered);
        }

        let loans = read_loans(&env, &borrower);
        let loan = loans
            .iter()
            .find(|loan| loan.id == loan_id)
            .ok_or(LPError::LoanNotFoundOrExists)?;

        Ok(loan.amount + calculate_fees(&env, &loan))
    }

    fn add_borrower(env: Env, borrower: Address) -> Result<(), LPError> {
        let admin = check_admin(&env)?;

        if has_borrower(&env, &borrower) {
            return Err(LPError::BorrowerAlreadyRegistered);
        }

        write_borrower(&env, &borrower, false);

        event::add_borrower(&env, admin, borrower);
        Ok(())
    }

    fn remove_borrower(env: Env, borrower: Address) -> Result<(), LPError> {
        let admin = check_admin(&env)?;

        if !has_borrower(&env, &borrower) {
            return Err(LPError::BorrowerNotRegistered);
        }

        remove_borrower(&env, &borrower);

        event::remove_borrower(&env, admin, borrower);
        Ok(())
    }

    fn add_lender(env: Env, lender: Address) -> Result<(), LPError> {
        let admin = check_admin(&env)?;

        if has_lender(&env, &lender) {
            return Err(LPError::LenderAlreadyRegistered);
        }

        write_lender(&env, &lender, &0i128);

        event::add_lender(&env, admin, lender);
        Ok(())
    }

    fn remove_lender(env: Env, lender: Address) -> Result<(), LPError> {
        let admin = check_admin(&env)?;

        if !has_lender(&env, &lender) {
            return Err(LPError::LenderNotRegistered);
        }

        remove_lender(&env, &lender);
        remove_lender_contribution(&env, &lender);

        event::remove_lender(&env, admin, lender);
        Ok(())
    }
}

mod test;
