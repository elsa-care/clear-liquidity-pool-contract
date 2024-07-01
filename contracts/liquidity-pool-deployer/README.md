# Deployer Smart Contract

It is a smart contract that allows deploying the liquidity pool contract for loans using USDC as the token.

## Users

#### Administrator:
- **Deploy Contract**: Deploys the liquidity pool contract.
- **Initialize Contract**: Initializes the liquidity pool contract with specific parameters.
- **Upload WASM**: Uploads the WASM code of the liquidity pool contract.
- **Invoke Contract**: Invokes specific functions within the deployed contract.

## Methods

#### deploy
 This function deploys a liquidity pool contract, setting the administrator, the token to be used, and a salt for the address of the deployed contract.
  - Params:
    - `env`: The execution environment of the contract.
    - `admin`: The address of the contract administrator.
    - `salt`: A 32-byte value used for the address of the deployed contract.
    - `token`: The address of the token (USDC).