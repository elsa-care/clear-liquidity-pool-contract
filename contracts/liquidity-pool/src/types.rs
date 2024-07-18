use soroban_sdk::{contracttype, Address, Map};

#[derive(Clone)]
#[contracttype]
pub struct Loan {
    pub id: u64,
    pub amount: i128,
    pub start_time: u64,
    pub contributions: Map<Address, i64>,
}

#[derive(Clone)]
#[contracttype]
pub struct Lender {
    pub active: bool,
    pub balance: i128,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TotalBalance,
    Token,
    Admin,
    Contribution,
    Borrower(Address),
    Lender(Address),
    Loan(Address),
}
