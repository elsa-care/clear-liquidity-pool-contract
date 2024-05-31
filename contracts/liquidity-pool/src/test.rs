#![cfg(test)]

use super::testutils::{create_token_contract, Setup};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_initialize() {
    let setup = Setup::new();

    let total_balance = setup.liquid_contract.client().get_total_balance();

    assert_eq!(total_balance, 0i128);
    assert_eq!(setup.liquid_contract.read_admin(), setup.admin);
    assert_eq!(setup.liquid_contract.read_token(), setup.token.address);
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
        .liquid_contract
        .client()
        .initialize(&admin, &token.address);
}

#[test]
fn test_add_borrower() {
    let setup = Setup::new();

    let borrower = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .add_borrower(&setup.admin, &borrower);
}

#[test]
#[should_panic(expected = "only the stored admin can add borrowers")]
fn test_add_borrower_with_fake_admin() {
    let setup = Setup::new();

    let fake_admin = Address::generate(&setup.env);
    let borrower = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .add_borrower(&fake_admin, &borrower);
}

#[test]
fn test_remove_borrower() {
    let setup = Setup::new();

    let borrower = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .remove_borrower(&setup.admin, &borrower);
}

#[test]
#[should_panic(expected = "only the stored admin can add borrowers")]
fn test_remove_borrower_with_fake_admin() {
    let setup = Setup::new();

    let fake_admin = Address::generate(&setup.env);
    let borrower = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .remove_borrower(&fake_admin, &borrower);
}

#[test]
fn test_add_lender() {
    let setup = Setup::new();

    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .add_lender(&setup.admin, &lender);

    assert!(setup.liquid_contract.has_lender(&lender));
}

#[test]
#[should_panic(expected = "only the stored admin can add lenders")]
fn test_add_lender_with_fake_admin() {
    let setup = Setup::new();

    let fake_admin = Address::generate(&setup.env);
    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .add_lender(&fake_admin, &lender);
}

#[test]
fn test_remove_lender() {
    let setup = Setup::new();

    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .add_lender(&setup.admin, &lender);

    assert!(setup.liquid_contract.has_lender(&lender));

    setup
        .liquid_contract
        .client()
        .remove_lender(&setup.admin, &lender);

    assert!(!setup.liquid_contract.has_lender(&lender));
}

#[test]
#[should_panic(expected = "only the stored admin can add lenders")]
fn test_remove_lender_with_fake_admin() {
    let setup = Setup::new();

    let fake_admin = Address::generate(&setup.env);
    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .remove_lender(&fake_admin, &lender);
}

#[test]
#[should_panic(expected = "lender is not registered")]
fn test_remove_without_lender() {
    let setup = Setup::new();

    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .remove_lender(&setup.admin, &lender);
}
