use soroban_sdk::{Address, Env, Vec};

use crate::types::DataKey;

pub fn get_admin(env: &Env) -> Address {
    env.storage().persistent().get(&DataKey::Admin).unwrap()
}

pub fn get_all_lenders(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::Lenders)
        .unwrap_or(Vec::new(&env))
}
