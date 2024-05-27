#![cfg(test)]

use crate::ClearContractClient;
use soroban_sdk::{
    testutils::Address as _,
    token::{self},
    Address, Env,
};

pub fn create_test_contract(
    env: &Env,
    admin: &Address,
    token: &Address,
) -> (Address, ClearContract) {
    let contract_id = register_test_contract(env);
    let contract = ClearContract::new(env, contract_id.clone());

    contract.client().initialize(&admin, &token);

    (contract_id, contract)
}

pub fn register_test_contract(env: &Env) -> Address {
    env.register_contract(None, crate::ClearContract {})
}

pub fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract(admin.clone());
    (
        token::Client::new(env, &contract_address),
        token::StellarAssetClient::new(env, &contract_address),
    )
}

pub struct Setup<'a> {
    pub env: Env,
    pub admin: Address,
    pub token: token::Client<'a>,
    pub clear_contract: ClearContract,
    pub clear_contract_id: Address,
}

pub struct ClearContract {
    env: Env,
    contract_id: Address,
}

impl Setup<'_> {
    pub fn new() -> Self {
        let env = Env::default();
        let admin = Address::generate(&env);
        let token_admin = Address::generate(&env);

        let (token, _token_client) = create_token_contract(&env, &token_admin);

        let (clear_contract_id, clear_contract) =
            create_test_contract(&env, &admin, &token.address);

        Self {
            env,
            admin,
            token,
            clear_contract,
            clear_contract_id,
        }
    }
}

impl ClearContract {
    #[must_use]
    pub fn client(&self) -> ClearContractClient {
        ClearContractClient::new(&self.env, &self.contract_id)
    }

    #[must_use]
    pub fn new(env: &Env, contract_id: Address) -> Self {
        Self {
            env: env.clone(),
            contract_id,
        }
    }
}
