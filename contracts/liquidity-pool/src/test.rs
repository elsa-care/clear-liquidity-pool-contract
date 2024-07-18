#![cfg(test)]
extern crate std;

use super::testutils::{create_token_contract, set_timestamp_for_20_days, Setup};
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    vec, Address, Env, IntoVal, Symbol,
};

#[test]
fn test_initialize() {
    let setup = Setup::new();
    let contract_events = setup.liquid_contract.get_contract_events();

    assert_eq!(setup.liquid_contract.read_contract_balance(), 0i128);
    assert_eq!(setup.liquid_contract.read_admin(), Ok(setup.admin.clone()));
    assert_eq!(
        setup.liquid_contract.read_token(),
        Ok(setup.token.address.clone())
    );
    assert_eq!(
        contract_events,
        vec![
            &setup.env,
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "initialize").as_val(),
                    setup.admin.into_val(&setup.env),
                    setup.token.address.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            )
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
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
fn test_balance_with_admin() {
    let setup = Setup::new();

    let balance = setup.liquid_contract.client().balance(&setup.admin);

    assert_eq!(setup.liquid_contract.read_contract_balance(), balance);
}

#[test]
fn test_balance_with_lender() {
    let setup = Setup::new();
    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender);

    let balance = setup.liquid_contract.client().balance(&lender);

    assert_eq!(setup.liquid_contract.read_lender(&lender), balance);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_balance_without_registered_address() {
    let setup = Setup::new();

    let unregistered_address = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .balance(&unregistered_address);
}

#[test]
fn test_deposit() {
    let setup = Setup::new();
    let lender1 = Address::generate(&setup.env);
    let lender2 = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender1);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender2);

    setup.token_admin.mock_all_auths().mint(&lender1, &4i128);
    setup.token_admin.mock_all_auths().mint(&lender2, &7i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .deposit(&lender1, &4i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .deposit(&lender2, &7i128);

    let contract_events = setup.liquid_contract.get_contract_events();

    assert_eq!(setup.liquid_contract.read_contract_balance(), 11i128);
    assert_eq!(setup.liquid_contract.read_lender(&lender1), 4i128);
    assert!(setup.liquid_contract.is_lender_in_contributions(&lender1));

    assert_eq!(setup.liquid_contract.read_lender(&lender2), 7i128);
    assert!(setup.liquid_contract.is_lender_in_contributions(&lender2));
    assert_eq!(
        contract_events,
        vec![
            &setup.env,
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "initialize").as_val(),
                    setup.admin.into_val(&setup.env),
                    setup.token.address.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_lender").as_val(),
                    setup.admin.into_val(&setup.env),
                    lender1.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_lender").as_val(),
                    setup.admin.into_val(&setup.env),
                    lender2.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "deposit").as_val(),
                    lender1.into_val(&setup.env),
                ],
                4i128.into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "deposit").as_val(),
                    lender2.into_val(&setup.env),
                ],
                7i128.into_val(&setup.env)
            )
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_deposit_without_lender() {
    let setup = Setup::new();
    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .deposit(&lender, &10i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_deposit_with_negative_amount() {
    let setup = Setup::new();
    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .deposit(&lender, &-10i128);
}

#[test]
fn test_withdraw() {
    let setup = Setup::new();
    let lender1 = Address::generate(&setup.env);
    let lender2 = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender1);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender2);

    setup.token_admin.mock_all_auths().mint(&lender1, &10i128);
    setup.token_admin.mock_all_auths().mint(&lender2, &10i128);
    setup
        .token_admin
        .mock_all_auths()
        .mint(&setup.liquid_contract_id, &20i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .deposit(&lender1, &10i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .deposit(&lender2, &10i128);

    assert_eq!(setup.liquid_contract.read_contract_balance(), 20i128);
    assert!(setup.liquid_contract.is_lender_in_contributions(&lender1));
    assert!(setup.liquid_contract.is_lender_in_contributions(&lender2));

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .withdraw(&lender1, &5i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .withdraw(&lender2, &7i128);

    let contract_events = setup.liquid_contract.get_contract_events();

    assert_eq!(setup.liquid_contract.read_contract_balance(), 8i128);
    assert_eq!(setup.liquid_contract.read_lender(&lender1), 5i128);
    assert_eq!(setup.liquid_contract.read_lender(&lender2), 3i128);
    assert!(setup.liquid_contract.is_lender_in_contributions(&lender1));
    assert!(setup.liquid_contract.is_lender_in_contributions(&lender2));
    assert_eq!(
        contract_events,
        vec![
            &setup.env,
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "initialize").as_val(),
                    setup.admin.into_val(&setup.env),
                    setup.token.address.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_lender").as_val(),
                    setup.admin.into_val(&setup.env),
                    lender1.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_lender").as_val(),
                    setup.admin.into_val(&setup.env),
                    lender2.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "deposit").as_val(),
                    lender1.into_val(&setup.env),
                ],
                10i128.into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "deposit").as_val(),
                    lender2.into_val(&setup.env),
                ],
                10i128.into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "withdraw").as_val(),
                    lender1.into_val(&setup.env),
                ],
                5i128.into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "withdraw").as_val(),
                    lender2.into_val(&setup.env),
                ],
                7i128.into_val(&setup.env)
            )
        ]
    );
}

#[test]
fn test_withdraw_by_remove_contribution() {
    let setup = Setup::new();
    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender);

    setup.token_admin.mock_all_auths().mint(&lender, &10i128);
    setup
        .token_admin
        .mock_all_auths()
        .mint(&setup.liquid_contract_id, &10i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .deposit(&lender, &10i128);

    assert_eq!(setup.liquid_contract.read_contract_balance(), 10i128);
    assert!(setup.liquid_contract.is_lender_in_contributions(&lender));

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .withdraw(&lender, &10i128);

    assert_eq!(setup.liquid_contract.read_contract_balance(), 0i128);
    assert_eq!(setup.liquid_contract.read_lender(&lender), 0i128);
    assert!(!setup.liquid_contract.is_lender_in_contributions(&lender));
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_withdraw_without_lender() {
    let setup = Setup::new();
    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .withdraw(&lender, &7i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_withdraw_negative_amount() {
    let setup = Setup::new();
    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender);

    setup.token_admin.mock_all_auths().mint(&lender, &7i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .deposit(&lender, &7i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .withdraw(&lender, &-7i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_withdraw_amount_greater_lender_balance() {
    let setup = Setup::new();
    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender);

    setup.token_admin.mock_all_auths().mint(&lender, &7i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .deposit(&lender, &7i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .withdraw(&lender, &10i128);
}

#[test]
fn test_loan() {
    let setup = Setup::new();
    let borrower = Address::generate(&setup.env);
    let lender1 = Address::generate(&setup.env);
    let lender2 = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender1);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender2);

    setup.token_admin.mock_all_auths().mint(&lender1, &10i128);
    setup.token_admin.mock_all_auths().mint(&lender2, &10i128);
    setup
        .token_admin
        .mock_all_auths()
        .mint(&setup.liquid_contract_id, &20i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .deposit(&lender1, &10i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .deposit(&lender2, &10i128);

    assert_eq!(setup.liquid_contract.read_contract_balance(), 20i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_borrower(&borrower);

    let loan_id = setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .loan(&borrower, &10i128);

    let contract_events = setup.liquid_contract.get_contract_events();

    assert_eq!(setup.liquid_contract.read_contract_balance(), 10i128);
    assert!(setup.liquid_contract.has_loan(&borrower, loan_id));
    assert_eq!(setup.liquid_contract.read_lender(&lender1), 5i128);
    assert_eq!(setup.liquid_contract.read_lender(&lender2), 5i128);
    assert_eq!(
        contract_events,
        vec![
            &setup.env,
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "initialize").as_val(),
                    setup.admin.into_val(&setup.env),
                    setup.token.address.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_lender").as_val(),
                    setup.admin.into_val(&setup.env),
                    lender1.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_lender").as_val(),
                    setup.admin.into_val(&setup.env),
                    lender2.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "deposit").as_val(),
                    lender1.into_val(&setup.env),
                ],
                10i128.into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "deposit").as_val(),
                    lender2.into_val(&setup.env),
                ],
                10i128.into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_borrower").as_val(),
                    setup.admin.into_val(&setup.env),
                    borrower.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "loan").as_val(),
                    borrower.into_val(&setup.env),
                    loan_id.into_val(&setup.env),
                ],
                10i128.into_val(&setup.env)
            )
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_loan_negative_amount() {
    let setup = Setup::new();
    let borrower = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_borrower(&borrower);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .loan(&borrower, &-10i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_loan_without_borrower() {
    let setup = Setup::new();
    let borrower = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .loan(&borrower, &10i128);
}

#[test]
fn test_request_two_loans() {
    let setup = Setup::new();
    let borrower = Address::generate(&setup.env);
    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender);

    setup.token_admin.mock_all_auths().mint(&lender, &20i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .deposit(&lender, &20i128);

    assert_eq!(setup.liquid_contract.read_contract_balance(), 20i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_borrower(&borrower);

    let first_loan_id = setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .loan(&borrower, &10i128);

    assert_eq!(setup.liquid_contract.read_contract_balance(), 10i128);
    assert!(setup.liquid_contract.has_loan(&borrower, first_loan_id));

    let second_loan_id = setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .loan(&borrower, &10i128);

    let contract_events = setup.liquid_contract.get_contract_events();

    assert_eq!(setup.liquid_contract.read_contract_balance(), 0i128);
    assert!(setup.liquid_contract.has_loan(&borrower, second_loan_id));
    assert_eq!(
        contract_events,
        vec![
            &setup.env,
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "initialize").as_val(),
                    setup.admin.into_val(&setup.env),
                    setup.token.address.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_lender").as_val(),
                    setup.admin.into_val(&setup.env),
                    lender.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "deposit").as_val(),
                    lender.into_val(&setup.env),
                ],
                20i128.into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_borrower").as_val(),
                    setup.admin.into_val(&setup.env),
                    borrower.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "loan").as_val(),
                    borrower.into_val(&setup.env),
                    first_loan_id.into_val(&setup.env),
                ],
                10i128.into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "loan").as_val(),
                    borrower.into_val(&setup.env),
                    second_loan_id.into_val(&setup.env),
                ],
                10i128.into_val(&setup.env)
            )
        ]
    );
}

#[test]
fn test_repay_loan_with_repayment_total_amount() {
    let setup = Setup::new();
    setup.env.mock_all_auths();

    let borrower = Address::generate(&setup.env);
    let lender1 = Address::generate(&setup.env);
    let lender2 = Address::generate(&setup.env);

    setup.liquid_contract.client().add_lender(&lender1);

    setup.liquid_contract.client().add_lender(&lender2);

    setup.token_admin.mint(&lender1, &5000i128);
    setup.token_admin.mint(&lender2, &5000i128);
    setup
        .token_admin
        .mint(&setup.liquid_contract_id, &10000i128);

    setup.liquid_contract.client().deposit(&lender1, &5000i128);

    setup.liquid_contract.client().deposit(&lender2, &5000i128);

    assert_eq!(setup.liquid_contract.read_contract_balance(), 10000i128);

    setup.liquid_contract.client().add_borrower(&borrower);

    let loan_id = setup.liquid_contract.client().loan(&borrower, &10000i128);

    assert_eq!(setup.liquid_contract.read_contract_balance(), 0i128);
    assert!(setup.liquid_contract.has_loan(&borrower, loan_id));
    assert_eq!(setup.liquid_contract.read_lender(&lender1), 0i128);
    assert_eq!(setup.liquid_contract.read_lender(&lender2), 0i128);

    set_timestamp_for_20_days(&setup.env);

    setup.token_admin.mint(&borrower, &10020i128);

    setup
        .liquid_contract
        .client()
        .repay_loan(&borrower, &loan_id, &10020i128);

    let contract_events = setup.liquid_contract.get_contract_events();

    assert_eq!(setup.liquid_contract.read_contract_balance(), 10020i128);
    assert_eq!(setup.liquid_contract.read_lender(&lender1), 5009i128);
    assert_eq!(setup.liquid_contract.read_lender(&lender2), 5009i128);
    assert!(!setup.liquid_contract.has_loan(&borrower, loan_id));
    assert_eq!(
        contract_events,
        vec![
            &setup.env,
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "initialize").as_val(),
                    setup.admin.into_val(&setup.env),
                    setup.token.address.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_lender").as_val(),
                    setup.admin.into_val(&setup.env),
                    lender1.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_lender").as_val(),
                    setup.admin.into_val(&setup.env),
                    lender2.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "deposit").as_val(),
                    lender1.into_val(&setup.env),
                ],
                5000i128.into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "deposit").as_val(),
                    lender2.into_val(&setup.env),
                ],
                5000i128.into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_borrower").as_val(),
                    setup.admin.into_val(&setup.env),
                    borrower.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "loan").as_val(),
                    borrower.into_val(&setup.env),
                    loan_id.into_val(&setup.env),
                ],
                10000i128.into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "repay_loan").as_val(),
                    borrower.into_val(&setup.env),
                    loan_id.into_val(&setup.env),
                ],
                10020i128.into_val(&setup.env)
            )
        ]
    );
}

#[test]
fn test_repay_loan_without_repayment_total_amount() {
    let setup = Setup::new();
    let borrower = Address::generate(&setup.env);
    let lender1 = Address::generate(&setup.env);
    let lender2 = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender1);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender2);

    setup.token_admin.mock_all_auths().mint(&lender1, &500i128);
    setup.token_admin.mock_all_auths().mint(&lender2, &500i128);
    setup
        .token_admin
        .mock_all_auths()
        .mint(&setup.liquid_contract_id, &1000i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .deposit(&lender1, &500i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .deposit(&lender2, &500i128);

    assert_eq!(setup.liquid_contract.read_contract_balance(), 1000i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_borrower(&borrower);

    let loan_id = setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .loan(&borrower, &1000i128);

    assert_eq!(setup.liquid_contract.read_contract_balance(), 0i128);
    assert!(setup.liquid_contract.has_loan(&borrower, loan_id));
    assert_eq!(setup.liquid_contract.read_lender(&lender1), 0i128);
    assert_eq!(setup.liquid_contract.read_lender(&lender2), 0i128);

    set_timestamp_for_20_days(&setup.env);

    setup
        .token_admin
        .mock_all_auths()
        .mint(&borrower, &1000i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .repay_loan(&borrower, &loan_id, &1000i128);

    let contract_events = setup.liquid_contract.get_contract_events();

    assert_eq!(setup.liquid_contract.read_contract_balance(), 1000i128);
    assert_eq!(setup.liquid_contract.read_lender(&lender1), 500i128);
    assert_eq!(setup.liquid_contract.read_lender(&lender2), 500i128);
    assert!(setup.liquid_contract.has_loan(&borrower, loan_id));
    assert_eq!(
        setup.liquid_contract.read_loan_amount(&borrower, loan_id),
        2i128
    );

    assert_eq!(
        contract_events,
        vec![
            &setup.env,
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "initialize").as_val(),
                    setup.admin.into_val(&setup.env),
                    setup.token.address.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_lender").as_val(),
                    setup.admin.into_val(&setup.env),
                    lender1.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_lender").as_val(),
                    setup.admin.into_val(&setup.env),
                    lender2.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "deposit").as_val(),
                    lender1.into_val(&setup.env),
                ],
                500i128.into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "deposit").as_val(),
                    lender2.into_val(&setup.env),
                ],
                500i128.into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_borrower").as_val(),
                    setup.admin.into_val(&setup.env),
                    borrower.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "loan").as_val(),
                    borrower.into_val(&setup.env),
                    loan_id.into_val(&setup.env),
                ],
                1000i128.into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "repay_loan").as_val(),
                    borrower.into_val(&setup.env),
                    loan_id.into_val(&setup.env),
                ],
                1000i128.into_val(&setup.env)
            )
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_repay_loan_negative_amount() {
    let setup = Setup::new();
    let borrower = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_borrower(&borrower);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .repay_loan(&borrower, &1u64, &-10i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_repay_loan_without_borrower() {
    let setup = Setup::new();
    let borrower = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .repay_loan(&borrower, &1u64, &10i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_repay_loan_without_active_loan() {
    let setup = Setup::new();
    let borrower = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_borrower(&borrower);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .repay_loan(&borrower, &1u64, &10i128);
}

#[test]
fn test_repay_loan_amount() {
    let setup = Setup::new();
    setup.env.mock_all_auths();
    let borrower = Address::generate(&setup.env);
    let lender = Address::generate(&setup.env);

    setup.liquid_contract.client().add_lender(&lender);

    setup.token_admin.mock_all_auths().mint(&lender, &1000i128);
    setup
        .token_admin
        .mock_all_auths()
        .mint(&setup.liquid_contract_id, &1000i128);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .deposit(&lender, &1000i128);

    assert_eq!(setup.liquid_contract.read_contract_balance(), 1000i128);

    setup.liquid_contract.client().add_borrower(&borrower);

    let loan_id = setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .loan(&borrower, &1000i128);

    assert!(setup.liquid_contract.has_loan(&borrower, loan_id));
    set_timestamp_for_20_days(&setup.env);

    let loan_amount = setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .repay_loan_amount(&borrower, &loan_id);

    assert_eq!(loan_amount, 1002i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_repay_loan_amount_without_borrower() {
    let setup = Setup::new();
    let borrower = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .repay_loan_amount(&borrower, &1u64);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_repay_loan_amount_without_active_loan() {
    let setup = Setup::new();
    setup.env.mock_all_auths();
    let borrower = Address::generate(&setup.env);

    setup.liquid_contract.client().add_borrower(&borrower);

    setup
        .liquid_contract
        .client()
        .repay_loan_amount(&borrower, &1u64);
}

#[test]
fn test_add_borrower() {
    let setup = Setup::new();
    let borrower = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_auths(&[MockAuth {
            address: &setup.admin,
            invoke: &MockAuthInvoke {
                contract: &setup.liquid_contract_id,
                fn_name: "add_borrower",
                args: (borrower.clone(),).into_val(&setup.env),
                sub_invokes: &[],
            },
        }])
        .add_borrower(&borrower);

    let contract_events = setup.liquid_contract.get_contract_events();

    assert!(setup.liquid_contract.has_borrower(&borrower));
    assert_eq!(
        contract_events,
        vec![
            &setup.env,
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "initialize").as_val(),
                    setup.admin.into_val(&setup.env),
                    setup.token.address.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_borrower").as_val(),
                    setup.admin.into_val(&setup.env),
                    borrower.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            )
        ]
    );
}

#[test]
#[should_panic = "Unauthorized function call for address"]
fn test_add_borrower_with_fake_admin() {
    let setup = Setup::new();
    let borrower = Address::generate(&setup.env);
    let fake_admin = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_auths(&[MockAuth {
            address: &fake_admin,
            invoke: &MockAuthInvoke {
                contract: &setup.liquid_contract_id,
                fn_name: "add_borrower",
                args: (borrower.clone(),).into_val(&setup.env),
                sub_invokes: &[],
            },
        }])
        .add_borrower(&borrower);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_add_registered_borrower() {
    let setup = Setup::new();

    let borrower = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_borrower(&borrower);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_borrower(&borrower);
}

#[test]
fn test_remove_borrower() {
    let setup = Setup::new();
    let borrower = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_borrower(&borrower);

    assert!(setup.liquid_contract.has_borrower(&borrower));

    setup
        .liquid_contract
        .client()
        .mock_auths(&[MockAuth {
            address: &setup.admin,
            invoke: &MockAuthInvoke {
                contract: &setup.liquid_contract_id,
                fn_name: "remove_borrower",
                args: (borrower.clone(),).into_val(&setup.env),
                sub_invokes: &[],
            },
        }])
        .remove_borrower(&borrower);

    let contract_events = setup.liquid_contract.get_contract_events();

    assert!(!setup.liquid_contract.has_borrower(&borrower));
    assert_eq!(
        contract_events,
        vec![
            &setup.env,
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "initialize").as_val(),
                    setup.admin.into_val(&setup.env),
                    setup.token.address.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_borrower").as_val(),
                    setup.admin.into_val(&setup.env),
                    borrower.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "remove_borrower").as_val(),
                    setup.admin.into_val(&setup.env),
                    borrower.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            )
        ]
    );
}

#[test]
#[should_panic = "Unauthorized function call for address"]
fn test_remove_borrower_with_fake_admin() {
    let setup = Setup::new();
    let borrower = Address::generate(&setup.env);
    let fake_admin = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_borrower(&borrower);

    assert!(setup.liquid_contract.has_borrower(&borrower));

    setup
        .liquid_contract
        .client()
        .mock_auths(&[MockAuth {
            address: &fake_admin,
            invoke: &MockAuthInvoke {
                contract: &setup.liquid_contract_id,
                fn_name: "remove_borrower",
                args: (borrower.clone(),).into_val(&setup.env),
                sub_invokes: &[],
            },
        }])
        .remove_borrower(&borrower);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_remove_without_borrower() {
    let setup = Setup::new();
    let borrower = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .remove_borrower(&borrower);
}

#[test]
fn test_add_lender() {
    let setup = Setup::new();
    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_auths(&[MockAuth {
            address: &setup.admin,
            invoke: &MockAuthInvoke {
                contract: &setup.liquid_contract_id,
                fn_name: "add_lender",
                args: (lender.clone(),).into_val(&setup.env),
                sub_invokes: &[],
            },
        }])
        .add_lender(&lender);

    let contract_events = setup.liquid_contract.get_contract_events();

    assert!(setup.liquid_contract.has_lender(&lender));
    assert_eq!(
        contract_events,
        vec![
            &setup.env,
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "initialize").as_val(),
                    setup.admin.into_val(&setup.env),
                    setup.token.address.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_lender").as_val(),
                    setup.admin.into_val(&setup.env),
                    lender.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            )
        ]
    );
}

#[test]
#[should_panic = "Unauthorized function call for address"]
fn test_add_lender_with_fake_admin() {
    let setup = Setup::new();
    let lender = Address::generate(&setup.env);
    let fake_admin = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_auths(&[MockAuth {
            address: &fake_admin,
            invoke: &MockAuthInvoke {
                contract: &setup.liquid_contract_id,
                fn_name: "add_lender",
                args: (lender.clone(),).into_val(&setup.env),
                sub_invokes: &[],
            },
        }])
        .add_lender(&lender);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_add_registered_lender() {
    let setup = Setup::new();
    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender);

    assert!(setup.liquid_contract.has_lender(&lender));

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender);
}

#[test]
fn test_remove_lender() {
    let setup = Setup::new();
    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender);

    assert!(setup.liquid_contract.has_lender(&lender));

    setup
        .liquid_contract
        .client()
        .mock_auths(&[MockAuth {
            address: &setup.admin,
            invoke: &MockAuthInvoke {
                contract: &setup.liquid_contract_id,
                fn_name: "remove_lender",
                args: (lender.clone(),).into_val(&setup.env),
                sub_invokes: &[],
            },
        }])
        .remove_lender(&lender);

    let contract_events = setup.liquid_contract.get_contract_events();

    assert!(!setup.liquid_contract.has_lender(&lender));
    assert_eq!(
        contract_events,
        vec![
            &setup.env,
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "initialize").as_val(),
                    setup.admin.into_val(&setup.env),
                    setup.token.address.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "add_lender").as_val(),
                    setup.admin.into_val(&setup.env),
                    lender.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            ),
            (
                setup.liquid_contract_id.clone(),
                vec![
                    &setup.env,
                    *Symbol::new(&setup.env, "remove_lender").as_val(),
                    setup.admin.into_val(&setup.env),
                    lender.into_val(&setup.env),
                ],
                ().into_val(&setup.env)
            )
        ]
    );
}

#[test]
#[should_panic = "Unauthorized function call for address"]
fn test_remove_lender_with_fake_admin() {
    let setup = Setup::new();
    let lender = Address::generate(&setup.env);
    let fake_admin = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .add_lender(&lender);

    assert!(setup.liquid_contract.has_lender(&lender));

    setup
        .liquid_contract
        .client()
        .mock_auths(&[MockAuth {
            address: &fake_admin,
            invoke: &MockAuthInvoke {
                contract: &setup.liquid_contract_id,
                fn_name: "remove_lender",
                args: (lender.clone(),).into_val(&setup.env),
                sub_invokes: &[],
            },
        }])
        .remove_lender(&lender);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_remove_without_lender() {
    let setup = Setup::new();
    let lender = Address::generate(&setup.env);

    setup
        .liquid_contract
        .client()
        .mock_all_auths()
        .remove_lender(&lender);
}
