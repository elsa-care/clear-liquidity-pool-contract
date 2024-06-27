use soroban_sdk::{Address, Env, Vec};

use crate::types::{DataKey, Loan};

pub fn has_admin(env: &Env) -> bool {
    env.storage().persistent().has(&DataKey::Admin)
}

pub fn read_admin(env: &Env) -> Address {
    env.storage().persistent().get(&DataKey::Admin).unwrap()
}

pub fn has_borrower(env: &Env, borrower: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Borrower(borrower.clone()))
}

pub fn has_loan(env: &Env, borrower: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Loan(borrower.clone()))
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

pub fn read_lender(env: &Env, lender: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Lender(lender.clone()))
        .unwrap_or(0)
}

pub fn read_token(env: &Env) -> Address {
    env.storage().persistent().get(&DataKey::Token).unwrap()
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

pub fn remove_lender_contribution(env: &Env, lender: &Address) {
    let mut contributions = read_contributions(env);

    if let Some(index) = contributions.iter().position(|address| &address == lender) {
        contributions.remove(index.try_into().unwrap());
    }

    write_lender_contribution(env, contributions);
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

pub fn write_loan(env: &Env, borrower: &Address, loan: &Loan) {
    env.storage()
        .persistent()
        .set(&DataKey::Loan(borrower.clone()), loan);
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
