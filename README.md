# Clear liquidity pool Contract

## Requirements

- Installing [Rust](https://www.rust-lang.org/tools/install)

- Install the wasm32-unknown-unknown target

  ```bash
  rustup target add wasm32-unknown-unknown
  ```

- Install the Soroban ClI
  ```bash
  cargo install --locked --version 21.0.1 soroban-cli
  ```
## How to run

Make sure you have soroban-cli installed, as explained above

### Option 1: Deploy on Futurenet

Deploy the contracts and initialize them

  ```bash
  ./initialize.sh futurenet
  ```

### Option 1: Deploy on Testnet

Deploy the contracts and initialize them

  ```bash
  ./initialize.sh testnet
  ```

### Option 3: Deploy on Localnet/Standalone

0. Run the soroban-rpc locally using the Stellar Quickstart Docker image

 
  ```bash
  docker-compose up -d
  ```

1. Keep that running, then deploy the contracts and initialize them:

  ```bash
  ./initialize.sh standalone
  ```

## Testing

Run tests with:
  ```bash
  cargo test
  ```