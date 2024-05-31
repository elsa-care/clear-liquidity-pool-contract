use soroban_sdk::{Address, Env, Vec};

use crate::types::DataKey;

pub fn has_admin(env: &Env) -> bool {
    env.storage().persistent().has(&DataKey::Admin)
}

pub fn read_admin(env: &Env) -> Address {
    env.storage().persistent().get(&DataKey::Admin).unwrap()
}

pub fn get_all_borrowers(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::Borrowers)
        .unwrap_or(Vec::new(&env))
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

pub fn read_lender(env: &Env, lender: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Lender(lender.clone()))
        .unwrap_or(0)
}

pub fn read_token(env: &Env) -> Address {
    env.storage().persistent().get(&DataKey::Token).unwrap()
}

pub fn remove_lender(env: &Env, lender: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::Lender(lender.clone()))
}

pub fn write_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
}

pub fn write_contract_balance(env: &Env, amount: &i128) {
    env.storage()
        .persistent()
        .set(&DataKey::TotalBalance, amount);
}

pub fn write_lender(env: &Env, lender: &Address, amount: &i128) {
    env.storage()
        .persistent()
        .set(&DataKey::Lender(lender.clone()), amount);
}

pub fn write_token(env: &Env, address: &Address) {
    env.storage().persistent().set(&DataKey::Token, address);
}
