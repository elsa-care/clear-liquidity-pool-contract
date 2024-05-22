use soroban_sdk::contracttype;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TotalBalance,
    Token,
    Admin,
}
