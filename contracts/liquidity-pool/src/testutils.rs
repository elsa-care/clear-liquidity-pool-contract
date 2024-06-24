#![cfg(test)]

use crate::storage::{
    has_borrower, has_lender, read_admin, read_contract_balance, read_lender, read_token,
};
use crate::LiquidityPoolContractClient;
use soroban_sdk::{
    testutils::Address as _,
    token::{self, StellarAssetClient},
    Address, Env,
};

pub fn create_test_contract(
    env: &Env,
    admin: &Address,
    token: &Address,
) -> (Address, LiquidityPoolContract) {
    let contract_id = register_test_contract(env);
    let contract = LiquidityPoolContract::new(env, contract_id.clone());

    contract.client().initialize(admin, token);

    (contract_id, contract)
}

pub fn register_test_contract(env: &Env) -> Address {
    env.register_contract(None, crate::LiquidityPoolContract {})
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
    pub token_admin: StellarAssetClient<'a>,
    pub liquid_contract: LiquidityPoolContract,
    pub liquid_contract_id: Address,
}

pub struct LiquidityPoolContract {
    env: Env,
    contract_id: Address,
}

impl Setup<'_> {
    pub fn new() -> Self {
        let env = Env::default();
        let admin = Address::generate(&env);
        let token_admin = Address::generate(&env);

        let (token, token_admin) = create_token_contract(&env, &token_admin);

        let (liquid_contract_id, liquid_contract) =
            create_test_contract(&env, &admin, &token.address);

        Self {
            env,
            admin,
            token,
            token_admin,
            liquid_contract,
            liquid_contract_id,
        }
    }
}

impl LiquidityPoolContract {
    #[must_use]
    pub fn client(&self) -> LiquidityPoolContractClient {
        LiquidityPoolContractClient::new(&self.env, &self.contract_id)
    }

    #[must_use]
    pub fn new(env: &Env, contract_id: Address) -> Self {
        Self {
            env: env.clone(),
            contract_id,
        }
    }

    pub fn read_admin(&self) -> Address {
        self.env
            .as_contract(&self.contract_id, || read_admin(&self.env))
    }

    pub fn has_borrower(&self, borrower: &Address) -> bool {
        self.env
            .as_contract(&self.contract_id, || has_borrower(&self.env, borrower))
    }

    pub fn has_lender(&self, lender: &Address) -> bool {
        self.env
            .as_contract(&self.contract_id, || has_lender(&self.env, lender))
    }

    pub fn read_token(&self) -> Address {
        self.env
            .as_contract(&self.contract_id, || read_token(&self.env))
    }

    pub fn read_contract_balance(&self) -> i128 {
        self.env
            .as_contract(&self.contract_id, || read_contract_balance(&self.env))
    }

    pub fn read_lender(&self, lender: &Address) -> i128 {
        self.env
            .as_contract(&self.contract_id, || read_lender(&self.env, lender))
    }
}
