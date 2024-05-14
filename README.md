# Clear liquidity pool Contract

## Requirements

- Installing [Rust](https://www.rust-lang.org/tools/install)

- Install the wasm32-unknown-unknown target

  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- Instal `soroban-cli` with this command:

  ```cargo install --locked soroban-cli --version 21.0.0-preview.1```


# How to run

  0.  Make sure you have soroban-cli installed, as explained above

  1. Deploy the contracts and initialize them with:

  ### Option 1: Deploy on Futurenet

  ```./initialize.sh futurenet```

  ### Option 2 : Deploy on Testnet

  ```./initialize.sh tesnet```


## Testing

Run tests with `cargo test`