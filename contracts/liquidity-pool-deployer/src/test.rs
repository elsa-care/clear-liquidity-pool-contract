#![cfg(test)]
extern crate alloc;
extern crate std;

use crate::{liquidity_pool, LiquidityPoolDeployer, LiquidityPoolDeployerClient};
use soroban_sdk::{
    testutils::Address as _,
    token::{self},
    Address, BytesN, Env,
};

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract(admin.clone());
    (
        token::Client::new(env, &contract_address),
        token::StellarAssetClient::new(env, &contract_address),
    )
}

#[test]
fn test_deploy_from_contract() {
    let env = Env::default();
    let deployer_client =
        LiquidityPoolDeployerClient::new(&env, &env.register_contract(None, LiquidityPoolDeployer));

    let salt = BytesN::from_array(&env, &[0; 32]);
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let vault = Address::generate(&env);

    let (token, _token_client) = create_token_contract(&env, &token_admin);

    env.mock_all_auths();
    let contract_id = deployer_client.deploy(&admin, &salt, &token.address, &vault);

    let client = liquidity_pool::Client::new(&env, &contract_id);
    let total_balance = client.balance(&admin);
    assert_eq!(total_balance, 0i128);
}
