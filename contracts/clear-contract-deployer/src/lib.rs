#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, IntoVal, Symbol, Val};

mod clear_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/clear_contract.wasm"
    );
}

#[contract]
pub struct ClearContractDeployer;

#[contractimpl]
impl ClearContractDeployer {
    pub fn deploy(env: Env, admin: Address, salt: BytesN<32>, token: Address) -> (Address, Val) {
        if admin != env.current_contract_address() {
            admin.require_auth();
        }

        let wasm_hash = env.deployer().upload_contract_wasm(clear_contract::WASM);

        let deployed_address = env.deployer().with_current_contract(salt).deploy(wasm_hash);

        let init_args = (admin, token).into_val(&env);

        let contract: Val = env.invoke_contract(
            &deployed_address,
            &Symbol::new(&env, "initialize"),
            init_args,
        );

        (deployed_address, contract)
    }
}
