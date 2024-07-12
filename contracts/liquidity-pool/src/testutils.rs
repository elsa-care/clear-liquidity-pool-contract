#![cfg(test)]

use crate::errors::LPError;
use crate::storage::{
    has_borrower, has_lender, read_admin, read_contract_balance, read_contributions, read_lender,
    read_loans, read_token,
};
use crate::LiquidityPoolContractClient;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{self, StellarAssetClient},
    Address, Env,
};

pub fn set_timestamp_for_20_days(env: &Env) {
    let initial_timestamp = env.ledger().timestamp();
    let days = 20;
    let seconds_per_day = 86400;
    let new_timestamp = initial_timestamp + (days * seconds_per_day) as u64;

    env.ledger().set_timestamp(new_timestamp)
}

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

    pub fn read_admin(&self) -> Result<Address, LPError> {
        self.env.as_contract(&self.contract_id, || {
            let admin = read_admin(&self.env)?;
            Ok(admin)
        })
    }

    pub fn has_borrower(&self, borrower: &Address) -> bool {
        self.env
            .as_contract(&self.contract_id, || has_borrower(&self.env, borrower))
    }

    pub fn has_loan(&self, borrower: &Address, loan_id: u64) -> bool {
        self.env.as_contract(&self.contract_id, || {
            let loans = read_loans(&self.env, borrower);
            loans.iter().any(|loan| loan.id == loan_id)
        })
    }

    pub fn has_lender(&self, lender: &Address) -> bool {
        self.env
            .as_contract(&self.contract_id, || has_lender(&self.env, lender))
    }

    pub fn read_token(&self) -> Result<Address, LPError> {
        self.env.as_contract(&self.contract_id, || {
            let token = read_token(&self.env)?;
            Ok(token)
        })
    }

    pub fn read_contract_balance(&self) -> i128 {
        self.env
            .as_contract(&self.contract_id, || read_contract_balance(&self.env))
    }

    pub fn read_loan_amount(&self, borrower: &Address, loan_id: u64) -> i128 {
        self.env.as_contract(&self.contract_id, || {
            let loans = read_loans(&self.env, borrower);
            let loan = loans.iter().find(|loan| loan.id == loan_id).unwrap();

            loan.amount
        })
    }

    pub fn read_lender(&self, lender: &Address) -> i128 {
        self.env
            .as_contract(&self.contract_id, || read_lender(&self.env, lender))
    }

    pub fn is_lender_in_contributions(&self, lender: &Address) -> bool {
        self.env.as_contract(&self.contract_id, || {
            let contributions = read_contributions(&self.env);
            contributions.iter().any(|address| address == *lender)
        })
    }
}
