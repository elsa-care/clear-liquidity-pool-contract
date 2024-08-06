#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, IntoVal, Symbol, Val};

mod liquidity_pool {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/liquidity_pool.wasm"
    );
}

#[contract]
pub struct LiquidityPoolDeployer;

#[contractimpl]
impl LiquidityPoolDeployer {
    pub fn deploy(
        env: Env,
        admin: Address,
        salt: BytesN<32>,
        token: Address,
        vault: Address,
    ) -> Address {
        if admin != env.current_contract_address() {
            admin.require_auth();
        }

        let wasm_hash = env.deployer().upload_contract_wasm(liquidity_pool::WASM);

        let deployed_address = env.deployer().with_current_contract(salt).deploy(wasm_hash);

        let init_args = (admin, token, vault).into_val(&env);

        let _contract: Val = env.invoke_contract(
            &deployed_address,
            &Symbol::new(&env, "initialize"),
            init_args,
        );

        deployed_address
    }
}

mod test;
