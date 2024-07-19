use soroban_sdk::{Address, Env, Symbol};

pub(crate) fn initialize(env: &Env, admin: Address, token: Address) {
    let topics = (Symbol::new(env, "initialize"), admin, token);
    env.events().publish(topics, ());
}

pub(crate) fn deposit(env: &Env, from: Address, amount: i128) {
    let topics = (Symbol::new(env, "deposit"), from);
    env.events().publish(topics, amount);
}

pub(crate) fn withdraw(env: &Env, to: Address, amount: i128) {
    let topics = (Symbol::new(env, "withdraw"), to);
    env.events().publish(topics, amount);
}

pub(crate) fn loan(env: &Env, to: Address, loan_id: u64, amount: i128) {
    let topics = (Symbol::new(env, "loan"), to, loan_id);
    env.events().publish(topics, amount);
}

pub(crate) fn repay_loan(env: &Env, to: Address, loan_id: u64, amount: i128) {
    let topics = (Symbol::new(env, "repay_loan"), to, loan_id);
    env.events().publish(topics, amount);
}

pub(crate) fn add_borrower(env: &Env, admin: Address, borrower: Address) {
    let topics = (Symbol::new(env, "add_borrower"), admin, borrower);
    env.events().publish(topics, ());
}

pub(crate) fn set_borrower_status(env: &Env, admin: Address, borrower: Address, active: bool) {
    let topics: (Symbol, Address, Address) =
        (Symbol::new(env, "set_borrower_status"), admin, borrower);
    env.events().publish(topics, active);
}

pub(crate) fn remove_borrower(env: &Env, admin: Address, borrower: Address) {
    let topics = (Symbol::new(env, "remove_borrower"), admin, borrower);
    env.events().publish(topics, ());
}

pub(crate) fn add_lender(env: &Env, admin: Address, lender: Address) {
    let topics = (Symbol::new(env, "add_lender"), admin, lender);
    env.events().publish(topics, ());
}

pub(crate) fn set_lender_status(env: &Env, admin: Address, lender: Address, active: bool) {
    let topics = (Symbol::new(env, "set_lender_status"), admin, lender);
    env.events().publish(topics, active);
}

pub(crate) fn remove_lender(env: &Env, admin: Address, lender: Address) {
    let topics = (Symbol::new(env, "remove_lender"), admin, lender);
    env.events().publish(topics, ());
}
