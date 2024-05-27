#![cfg(test)]

use super::testutils::{create_token_contract, Setup};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_initialize() {
    let setup = Setup::new();

    let total_balance = setup.clear_contract.client().get_total_balance();

    assert_eq!(total_balance, 0i128);
}

#[test]
#[should_panic(expected = "contract already initialized with an admin")]
fn test_already_initialize() {
    let setup = Setup::new();

    let env = Env::default();
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token, _token_client) = create_token_contract(&env, &token_admin);

    setup
        .clear_contract
        .client()
        .initialize(&admin, &token.address);
}
