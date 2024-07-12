use soroban_sdk::{Address, Env, Vec};

use crate::{
    errors::LPError,
    types::{DataKey, Loan},
};

pub fn check_admin(env: &Env) -> Result<(), LPError> {
    let admin = read_admin(env)?;
    admin.require_auth();
    Ok(())
}

pub fn has_admin(env: &Env) -> bool {
    env.storage().persistent().has(&DataKey::Admin)
}

pub fn read_admin(env: &Env) -> Result<Address, LPError> {
    match env.storage().persistent().get(&DataKey::Admin) {
        Some(admin) => Ok(admin),
        None => Err(LPError::AdminNotFound),
    }
}

pub fn has_borrower(env: &Env, borrower: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Borrower(borrower.clone()))
}

pub fn has_lender(env: &Env, lender: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Lender(lender.clone()))
}

pub fn read_contract_balance(env: &Env) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::TotalBalance)
        .unwrap_or(0)
}

pub fn read_contributions(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::Contribution)
        .unwrap_or(Vec::new(env))
}

pub fn read_loans(env: &Env, borrower: &Address) -> Vec<Loan> {
    env.storage()
        .persistent()
        .get(&DataKey::Loan(borrower.clone()))
        .unwrap_or(Vec::new(env))
}

pub fn read_lender(env: &Env, lender: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Lender(lender.clone()))
        .unwrap_or(0)
}

pub fn read_token(env: &Env) -> Result<Address, LPError> {
    match env.storage().persistent().get(&DataKey::Token) {
        Some(token) => Ok(token),
        None => Err(LPError::TokenNotFound),
    }
}

pub fn remove_borrower(env: &Env, borrower: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::Borrower(borrower.clone()))
}

pub fn remove_lender(env: &Env, lender: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::Lender(lender.clone()))
}

pub fn remove_lender_contribution(env: &Env, lender: &Address) -> Result<(), LPError> {
    let mut contributions = read_contributions(env);

    if let Some(index) = contributions.iter().position(|address| &address == lender) {
        match index.try_into() {
            Ok(valid_index) => contributions.remove(valid_index),
            Err(_) => return Err(LPError::LenderNotFoundInContributions),
        };
    }

    write_lender_contribution(env, contributions);
    Ok(())
}

pub fn write_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
}

pub fn write_borrower(env: &Env, borrower: &Address, is_loaned: bool) {
    env.storage()
        .persistent()
        .set(&DataKey::Borrower(borrower.clone()), &is_loaned);
}

pub fn write_contract_balance(env: &Env, amount: &i128) {
    env.storage()
        .persistent()
        .set(&DataKey::TotalBalance, amount);
}

pub fn write_loans(env: &Env, borrower: &Address, loans: &Vec<Loan>) {
    env.storage()
        .persistent()
        .set(&DataKey::Loan(borrower.clone()), loans);
}

pub fn write_lender(env: &Env, lender: &Address, amount: &i128) {
    env.storage()
        .persistent()
        .set(&DataKey::Lender(lender.clone()), amount);
}

pub fn write_lender_contribution(env: &Env, contributions: Vec<Address>) {
    env.storage()
        .persistent()
        .set(&DataKey::Contribution, &contributions);
}

pub fn write_token(env: &Env, address: &Address) {
    env.storage().persistent().set(&DataKey::Token, address);
}
