#![cfg(test)]
extern crate alloc;
extern crate std;

use crate::{clear_contract, ClearContractDeployer, ClearContractDeployerClient};
use soroban_sdk::{
  symbol_short,
  testutils::Address as _,
  token::{self},
  Address, BytesN, Env
}

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
  let deployer_client = ClearContractDeployerClient::new(
    &env,
    &env.register_contract(None, ClearContractDeployer),
  );

  let salt = BytesN::from_array(&env, &[0; 32]);
  let admin = Address::generate(&env);
  let (token, _token_client) = create_token_contract(&env, &token_admin);
  let init_args = (admin, token).into_val(&env);

  env.mock_all_auths();
  let (contract_id, _contract) = deployer_client.deploy(
    &admin,
    &salt,
    &init_args,
  );

  let client = clear_contract::Client::new(&env, &contract_id);
  let total_balance = client.get_total_balance();
  assert_eq!(total_balance, 0i128);
}