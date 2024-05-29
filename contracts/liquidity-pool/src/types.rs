use soroban_sdk::contracttype;

#[derive(Clone, Copy)]
#[contracttype]
pub enum DataKey {
    TotalBalance,
    Token,
    Admin,
    Lenders,
}
