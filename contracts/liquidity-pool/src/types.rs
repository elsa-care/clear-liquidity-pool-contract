use soroban_sdk::{contracttype, Address};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TotalBalance,
    Token,
    Admin,
    LenderContribution,
    Borrower(Address),
    Lender(Address),
}
